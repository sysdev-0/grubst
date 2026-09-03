use std::fs;
use std::path::Path;

/// GRUB configuration files that we modify
const GRUB_CONFIG_FILES: &[&str] = &[
    "/etc/grub.d/01_grubst_users",
    "/etc/grub.d/02_grubst_usbcheck",
    "/etc/grub.d/99_grubst_guard",
    "/etc/default/grub",
    "/boot/grub/grub.cfg",
];

/// Backup directory
const BACKUP_DIR: &str = "/var/lib/grubst/backups";

/// Create a timestamped backup of all GRUB config files
pub fn create_backup() -> Result<String, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = Path::new(BACKUP_DIR).join(&timestamp);

    fs::create_dir_all(&backup_path)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Backup /etc/grub.d/ directory
    let grub_d = Path::new("/etc/grub.d");
    if grub_d.exists() {
        let grub_d_backup = backup_path.join("grub.d");
        fs::create_dir_all(&grub_d_backup)
            .map_err(|e| format!("Failed to create grub.d backup dir: {}", e))?;

        if let Ok(entries) = fs::read_dir(grub_d) {
            for entry in entries.flatten() {
                let source = entry.path();
                if source.is_file() {
                    let dest = grub_d_backup.join(entry.file_name());
                    if let Err(e) = fs::copy(&source, &dest) {
                        log::warn!("Failed to backup {:?}: {}", source, e);
                    }
                }
            }
        }
    }

    // Backup /etc/default/grub
    let default_grub = Path::new("/etc/default/grub");
    if default_grub.exists() {
        let dest = backup_path.join("default_grub");
        fs::copy(default_grub, &dest)
            .map_err(|e| format!("Failed to backup /etc/default/grub: {}", e))?;
    }

    // Backup /boot/grub/grub.cfg
    let grub_cfg = Path::new("/boot/grub/grub.cfg");
    if grub_cfg.exists() {
        let dest = backup_path.join("grub.cfg");
        fs::copy(grub_cfg, &dest)
            .map_err(|e| format!("Failed to backup grub.cfg: {}", e))?;
    }

    // Write backup metadata
    let metadata = serde_json::json!({
        "timestamp": timestamp,
        "created_at": chrono::Local::now().to_rfc3339(),
        "files_backed_up": GRUB_CONFIG_FILES,
    });
    let metadata_path = backup_path.join("metadata.json");
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .map_err(|e| format!("Failed to write backup metadata: {}", e))?;

    // Set restrictive permissions on backup directory
    set_dir_permissions(&backup_path)?;

    log::info!("Backup created: {:?}", backup_path);
    Ok(timestamp)
}

/// Restore GRUB configuration from the latest backup
pub fn restore_backup() -> Result<(), String> {
    let backup_dir = Path::new(BACKUP_DIR);
    if !backup_dir.exists() {
        return Err("No backups found".to_string());
    }

    // Find the latest backup (alphabetically sorted timestamps)
    let mut backups: Vec<_> = fs::read_dir(backup_dir)
        .map_err(|e| format!("Failed to read backup dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();

    backups.sort();

    let latest = backups
        .last()
        .ok_or_else(|| "No backup directories found".to_string())?;

    log::info!("Restoring from backup: {:?}", latest);

    // Restore /etc/grub.d/
    let grub_d_backup = latest.join("grub.d");
    if grub_d_backup.exists() {
        let grub_d = Path::new("/etc/grub.d");

        // Only restore original system files (not GRUBST files)
        if let Ok(entries) = fs::read_dir(&grub_d_backup) {
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                // Don't restore GRUBST-specific scripts (they should be removed)
                if filename.contains("grubst") {
                    continue;
                }

                let source = entry.path();
                let dest = grub_d.join(&filename);

                if let Err(e) = fs::copy(&source, &dest) {
                    log::warn!("Failed to restore {:?}: {}", dest, e);
                }
            }
        }
    }

    // Restore /etc/default/grub
    let default_grub_backup = latest.join("default_grub");
    if default_grub_backup.exists() {
        fs::copy(&default_grub_backup, "/etc/default/grub")
            .map_err(|e| format!("Failed to restore /etc/default/grub: {}", e))?;
    }

    // Restore /boot/grub/grub.cfg (if it was backed up)
    let grub_cfg_backup = latest.join("grub.cfg");
    if grub_cfg_backup.exists() {
        fs::copy(&grub_cfg_backup, "/boot/grub/grub.cfg")
            .map_err(|e| format!("Failed to restore /boot/grub/grub.cfg: {}", e))?;
    }

    log::info!("Backup restored successfully");
    Ok(())
}

/// List all available backups
pub fn list_backups() -> Vec<BackupInfo> {
    let backup_dir = Path::new(BACKUP_DIR);
    if !backup_dir.exists() {
        return Vec::new();
    }

    let mut backups = Vec::new();

    if let Ok(entries) = fs::read_dir(backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                let timestamp = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let created_at = if metadata_path.exists() {
                    fs::read_to_string(&metadata_path)
                        .ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                        .and_then(|v| v.get("created_at").and_then(|v| v.as_str()).map(String::from))
                        .unwrap_or(timestamp.clone())
                } else {
                    timestamp.clone()
                };

                backups.push(BackupInfo {
                    timestamp,
                    created_at,
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    backups
}

/// Set restrictive permissions on backup directory (root-only)
fn set_dir_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perms)
        .map_err(|e| format!("Failed to set permissions: {}", e))?;
    Ok(())
}

/// Information about a backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub timestamp: String,
    pub created_at: String,
    pub path: String,
}
