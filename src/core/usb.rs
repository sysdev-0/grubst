use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents a detected USB device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    /// Device path (e.g., /dev/sdb)
    pub path: String,
    /// Human-readable device name
    pub name: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Human-readable size (e.g., "16 GB")
    pub size_human: String,
    /// Device model
    pub model: String,
    /// Vendor
    pub vendor: String,
}

/// Minimum required USB size: 500 MB (lowered for testing)
const MIN_USB_SIZE: u64 = 500_000_000;

/// Hidden directory name on the USB for GRUBST data
const GRUBST_DIR: &str = ".grubst";
/// Auth token filename
const AUTH_TOKEN_FILE: &str = "auth.token";
/// Machine fingerprint filename
const FINGERPRINT_FILE: &str = "machine.fp";
/// Integrity hash filename
const INTEGRITY_FILE: &str = "integrity.sha512";
/// GRUB-readable token script (sourced by GRUB)
const TOKEN_CFG_FILE: &str = "token.cfg";
/// GRUB-readable machine fingerprint script (sourced by GRUB)
const MACHINE_CFG_FILE: &str = "machine.cfg";
/// GRUBST marker filename
const MARKER_FILE: &str = "GRUBST_KEY";

fn root_block_device() -> Option<String> {
    let mounts = fs::read_to_string("/proc/self/mounts").ok()?;
    let root_line = mounts.lines().find(|l| l.split_whitespace().nth(1) == Some("/"))?;
    let source = root_line.split_whitespace().next()?;
    if !source.starts_with("/dev/") {
        return None;
    }
    if source.starts_with("/dev/mapper/") {
        return None;
    }
    Some(strip_partition_suffix(source))
}

fn strip_partition_suffix(dev: &str) -> String {
    let basename = dev.rsplit('/').next().unwrap_or(dev);

    // NVMe/MMC style: /dev/nvme0n1p2 → /dev/nvme0n1
    if basename.starts_with("nvme") || basename.starts_with("mmcblk") {
        if let Some(idx) = basename.rfind('p') {
            let after = &basename[idx + 1..];
            if !after.is_empty() && after.chars().all(|c| c.is_ascii_digit()) {
                let prefix_len = dev.len() - basename.len() + idx;
                return dev[..prefix_len].to_string();
            }
        }
        return dev.to_string();
    }

    // SCSI/SATA/VirtIO style: /dev/sda1 → /dev/sda
    if basename.starts_with("sd") || basename.starts_with("hd") || basename.starts_with("vd") {
        let mut base = dev.to_string();
        while base.chars().last().is_some_and(|c| c.is_ascii_digit()) {
            base.pop();
        }
        return base;
    }

    // Other devices (loop, etc.): return as-is
    dev.to_string()
}

/// Detect all connected USB block devices
pub fn detect_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    let root_dev = root_block_device();

    // Use lsblk to find removable block devices
    let output = match Command::new("lsblk")
        .args([
            "-J",        // JSON output
            "-b",        // bytes
            "-d",        // no partitions
            "-o", "NAME,SIZE,MODEL,VENDOR,RM,TRAN,TYPE",
        ])
        .output()
    {
        Ok(o) => o,
        Err(_) => return devices,
    };

    if !output.status.success() {
        return devices;
    }

    let json_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return devices,
    };

    // Parse lsblk JSON
    let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return devices,
    };

    if let Some(blockdevices) = parsed.get("blockdevices").and_then(|v| v.as_array()) {
        for dev in blockdevices {
            let rm = dev.get("rm").and_then(|v| v.as_bool()).unwrap_or(false);
            // Also check if rm is returned as a string "1" or number 1
            let rm_alt = dev
                .get("rm")
                .map(|v| {
                    v.as_str() == Some("1")
                        || v.as_u64() == Some(1)
                        || v.as_bool() == Some(true)
                })
                .unwrap_or(false);

            let tran = dev
                .get("tran")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let dev_type = dev
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Filter: must be removable OR USB transport, and type must be disk
            if (rm || rm_alt || tran == "usb") && dev_type == "disk" {
                let name_str = dev
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let device_path = format!("/dev/{}", name_str);
                if root_dev.as_deref() == Some(device_path.as_str()) {
                    continue;
                }

                let size_bytes = dev
                    .get("size")
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .unwrap_or(0);

                let model = dev
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();

                let vendor = dev
                    .get("vendor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let display_name = if model.is_empty() || model == "Unknown" {
                    format!("USB Drive ({})", name_str)
                } else if vendor.is_empty() {
                    model.clone()
                } else {
                    format!("{} {}", vendor, model)
                };

                devices.push(UsbDevice {
                    path: device_path,
                    name: display_name,
                    size_bytes,
                    size_human: format_size(size_bytes),
                    model,
                    vendor,
                });
            }
        }
    }

    devices
}

/// Format USB drive as FAT32
pub fn format_usb(device: &UsbDevice) -> Result<(), String> {
    log::info!("Formatting USB device: {}", device.path);

    if device.size_bytes < MIN_USB_SIZE {
        return Err(format!(
            "USB device too small: {} (minimum 500 MB required)",
            device.size_human
        ));
    }

    // Step 1: Unmount any mounted partitions
    let _ = Command::new("umount")
        .arg(format!("{}1", device.path))
        .output();
    let _ = Command::new("umount")
        .arg(&device.path)
        .output();

    // Small delay for unmount
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Step 2: Wipe partition table
    let output = Command::new("wipefs")
        .args(["--all", "--force", &device.path])
        .output()
        .map_err(|e| format!("Failed to run wipefs: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wipefs failed: {}", stderr));
    }

    // Step 3: Create new partition table (MBR) and single FAT32 partition
    let fdisk_script = "o\nn\np\n1\n\n\nt\nc\nw\n";
    let mut fdisk = Command::new("fdisk")
        .arg(&device.path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run fdisk: {}", e))?;

    if let Some(ref mut stdin) = fdisk.stdin {
        use std::io::Write;
        let _ = stdin.write_all(fdisk_script.as_bytes());
    }

    let _fdisk_result = fdisk
        .wait()
        .map_err(|e| format!("fdisk error: {}", e))?;

    // fdisk may exit with non-zero even on success, so we continue

    // Small delay for kernel to re-read partition table
    let _ = Command::new("partprobe").arg(&device.path).output();
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Step 4: Format partition as FAT32
    let partition_path = format!("{}1", device.path);

    if !Path::new(&partition_path).exists() {
        return Err(format!(
            "USB partition not found after repartitioning: {}",
            partition_path
        ));
    }

    let output = Command::new("mkfs.vfat")
        .args(["-F", "32", "-n", "GRUBST_KEY", &partition_path])
        .output()
        .map_err(|e| format!("Failed to run mkfs.vfat: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mkfs.vfat failed: {}", stderr));
    }

    log::info!("USB formatted successfully: {}", partition_path);
    Ok(())
}

/// Write authentication data to the USB drive
pub fn write_auth_data(
    device: &UsbDevice,
    auth_token: &str,
    fingerprint: &str,
) -> Result<(), String> {
    let partition_path = format!("{}1", device.path);
    let mount_dir = tempfile::Builder::new()
        .prefix("grubst_usb_")
        .tempdir()
        .map_err(|e| format!("Failed to create temp mount dir: {}", e))?;
    let mount_point = mount_dir.path().to_string_lossy().to_string();

    // Create mount point
    fs::create_dir_all(&mount_point).map_err(|e| format!("Failed to create mount point: {}", e))?;

    // Mount the partition
    let output = Command::new("mount")
        .args([&partition_path, &mount_point])
        .output()
        .map_err(|e| format!("Failed to mount USB: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Mount failed: {}", stderr));
    }

    // Create GRUBST directory
    let grubst_path = Path::new(&mount_point).join(GRUBST_DIR);
    fs::create_dir_all(&grubst_path)
        .map_err(|e| format!("Failed to create GRUBST directory: {}", e))?;

    let token_cfg_path = grubst_path.join(TOKEN_CFG_FILE);
    let token_cfg_content = format!("set grubst_token=\"{}\"\n", auth_token);
    fs::write(&token_cfg_path, token_cfg_content)
        .map_err(|e| format!("Failed to write token cfg: {}", e))?;

    // Write auth token
    let auth_path = grubst_path.join(AUTH_TOKEN_FILE);
    fs::write(&auth_path, auth_token)
        .map_err(|e| format!("Failed to write auth token: {}", e))?;

    // Write machine fingerprint
    let fp_path = grubst_path.join(FINGERPRINT_FILE);
    fs::write(&fp_path, fingerprint)
        .map_err(|e| format!("Failed to write fingerprint: {}", e))?;

    let machine_cfg_path = grubst_path.join(MACHINE_CFG_FILE);
    let machine_cfg_content = format!("set grubst_machine_fp_usb=\"{}\"\n", fingerprint);
    fs::write(&machine_cfg_path, machine_cfg_content)
        .map_err(|e| format!("Failed to write machine cfg: {}", e))?;

    // Write marker file (GRUB will look for this)
    let marker_path = Path::new(&mount_point).join(MARKER_FILE);
    fs::write(&marker_path, "GRUBST_RESCUE_KEY_V1\n")
        .map_err(|e| format!("Failed to write marker: {}", e))?;

    // Calculate and write integrity hash
    let auth_data = fs::read(&auth_path).unwrap_or_default();
    let fp_data = fs::read(&fp_path).unwrap_or_default();
    let mut combined = auth_data;
    combined.extend_from_slice(&fp_data);
    let integrity = crate::core::crypto::integrity_hash(&combined);

    let integrity_path = grubst_path.join(INTEGRITY_FILE);
    fs::write(&integrity_path, &integrity)
        .map_err(|e| format!("Failed to write integrity hash: {}", e))?;

    // Sync and unmount
    let _ = Command::new("sync").output();
    std::thread::sleep(std::time::Duration::from_millis(500));

    let output = Command::new("umount")
        .arg(&mount_point)
        .output()
        .map_err(|e| format!("Failed to unmount: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("Unmount warning: {}", stderr);
    }

    // Get the UUID of the formatted partition for GRUB config
    log::info!("Auth data written to USB successfully");
    Ok(())
}

/// Get the UUID of a partition
pub fn get_partition_uuid(device_path: &str) -> Result<String, String> {
    let partition = format!("{}1", device_path);
    let output = Command::new("blkid")
        .args(["-s", "UUID", "-o", "value", &partition])
        .output()
        .map_err(|e| format!("blkid failed: {}", e))?;

    if !output.status.success() {
        return Err("blkid returned error".to_string());
    }

    let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uuid.is_empty() {
        return Err("No UUID found".to_string());
    }

    Ok(uuid)
}

/// Format bytes into human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1_000;
    const MB: u64 = 1_000_000;
    const GB: u64 = 1_000_000_000;
    const TB: u64 = 1_000_000_000_000;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_partition_suffix() {
        // NVMe drives
        assert_eq!(strip_partition_suffix("/dev/nvme0n1p1"), "/dev/nvme0n1");
        assert_eq!(strip_partition_suffix("/dev/nvme0n1p2"), "/dev/nvme0n1");
        assert_eq!(strip_partition_suffix("/dev/nvme1n1"), "/dev/nvme1n1");
        
        // SATA/SCSI drives
        assert_eq!(strip_partition_suffix("/dev/sda1"), "/dev/sda");
        assert_eq!(strip_partition_suffix("/dev/sdb2"), "/dev/sdb");
        assert_eq!(strip_partition_suffix("/dev/sdc"), "/dev/sdc");
        
        // MMC drives
        assert_eq!(strip_partition_suffix("/dev/mmcblk0p1"), "/dev/mmcblk0");
        
        // Loop devices or others should remain unaffected
        assert_eq!(strip_partition_suffix("/dev/loop0"), "/dev/loop0");
        assert_eq!(strip_partition_suffix("/dev/mapper/vg-lv"), "/dev/mapper/vg-lv");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1_500), "1.5 KB");
        assert_eq!(format_size(2_500_000), "2.5 MB");
        assert_eq!(format_size(4_200_000_000), "4.2 GB");
    }
}
