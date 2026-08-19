use sha2::{Digest, Sha512};
use std::fs;

/// Generate a unique machine fingerprint by combining hardware identifiers.
/// This fingerprint is used to bind the USB rescue key to this specific machine.
pub fn generate_fingerprint() -> String {
    let mut components: Vec<String> = Vec::new();

    // 1. Machine ID (systemd)
    if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
        components.push(format!("machine_id:{}", machine_id.trim()));
    }

    // 2. DMI product UUID (motherboard)
    if let Ok(product_uuid) = fs::read_to_string("/sys/class/dmi/id/product_uuid") {
        components.push(format!("product_uuid:{}", product_uuid.trim()));
    }

    // 3. Board serial
    if let Ok(board_serial) = fs::read_to_string("/sys/class/dmi/id/board_serial") {
        components.push(format!("board_serial:{}", board_serial.trim()));
    }

    // 4. DMI product serial
    if let Ok(product_serial) = fs::read_to_string("/sys/class/dmi/id/product_serial") {
        components.push(format!("product_serial:{}", product_serial.trim()));
    }

    // 5. CPU info (model name as fallback)
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                components.push(format!("cpu:{}", line.trim()));
                break;
            }
        }
    }

    // 6. Root disk serial (boot disk)
    if let Ok(entries) = fs::read_dir("/dev/disk/by-id/") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Look for the main disk (not partitions)
            if (name.starts_with("ata-") || name.starts_with("nvme-"))
                && !name.contains("-part")
            {
                components.push(format!("disk:{}", name));
                break;
            }
        }
    }

    // If we got nothing, use a fallback
    if components.is_empty() {
        components.push("fallback:unknown_machine".to_string());
    }

    // Combine all components and hash
    let combined = components.join("|");
    let mut hasher = Sha512::new();
    hasher.update(b"GRUBST_FINGERPRINT_V1::");
    hasher.update(combined.as_bytes());

    hex::encode(hasher.finalize())
}

/// Verify that a stored fingerprint matches the current machine
pub fn verify_fingerprint(stored: &str) -> bool {
    let current = generate_fingerprint();
    current == stored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_deterministic() {
        let fp1 = generate_fingerprint();
        let fp2 = generate_fingerprint();
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_not_empty() {
        let fp = generate_fingerprint();
        assert!(!fp.is_empty());
        assert!(fp.len() == 128); // SHA-512 hex = 128 chars
    }
}
