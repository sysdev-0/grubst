use std::fs;
use std::path::Path;
use std::process::Command;

/// Severity level of an audit finding
#[derive(Debug, Clone, PartialEq)]
pub enum AuditSeverity {
    /// Critical — immediate action needed
    Critical,
    /// Warning — should be addressed
    Warning,
    /// Informational — good to know
    Info,
    /// Pass — check passed
    Pass,
}

/// A single audit finding
#[derive(Debug, Clone)]
pub struct AuditItem {
    pub title: String,
    pub description: String,
    pub severity: AuditSeverity,
    pub fix_hint: Option<String>,
}

/// Full audit report
#[derive(Debug, Clone)]
pub struct AuditReport {
    pub score: u32,
    pub summary: String,
    pub items: Vec<AuditItem>,
}

/// Run a comprehensive security audit of the system
pub fn run_audit() -> AuditReport {
    let mut items = Vec::new();

    // ── Check 1: GRUB password protection ──
    items.push(check_grub_password());

    // ── Check 2: GRUB config file permissions ──
    items.push(check_grub_permissions());

    // ── Check 3: Full disk encryption (LUKS) ──
    items.push(check_disk_encryption());

    // ── Check 4: Secure Boot status ──
    items.push(check_secure_boot());

    // ── Check 5: BIOS/UEFI boot order protection ──
    items.push(check_boot_order());

    // ── Check 6: OS password policy ──
    items.push(check_root_password());

    // ── Check 7: GRUBST update guard ──
    items.push(check_update_guard());

    // ── Check 8: Backup status ──
    items.push(check_backup_status());

    // Calculate score
    let total = items.len() as u32;
    let passed = items
        .iter()
        .filter(|i| i.severity == AuditSeverity::Pass)
        .count() as u32;
    let critical = items
        .iter()
        .filter(|i| i.severity == AuditSeverity::Critical)
        .count() as u32;

    let score = if total > 0 {
        let base = (passed * 100) / total;
        // Penalize heavily for critical issues
        base.saturating_sub(critical * 15)
    } else {
        0
    };

    let summary = if score >= 80 {
        "Your system has strong boot security.".to_string()
    } else if score >= 50 {
        "Your boot security needs improvement.".to_string()
    } else {
        "Your boot security is weak — action recommended.".to_string()
    };

    AuditReport {
        score,
        summary,
        items,
    }
}

// ─────────────────────────── Individual Checks ───────────────────────────

fn check_grub_password() -> AuditItem {
    let superuser_script = Path::new("/etc/grub.d/01_grubst_users");
    let grub_cfg = Path::new("/boot/grub/grub.cfg");

    // Check if GRUBST protection is active
    if superuser_script.exists() {
        return AuditItem {
            title: "GRUB Password Protection".to_string(),
            description: "GRUBST protection is active. Bootloader editing requires authentication."
                .to_string(),
            severity: AuditSeverity::Pass,
            fix_hint: None,
        };
    }

    // Check if grub.cfg has any password config
    if grub_cfg.exists() {
        if let Ok(content) = fs::read_to_string(grub_cfg) {
            if content.contains("superusers") || content.contains("password_pbkdf2") {
                return AuditItem {
                    title: "GRUB Password Protection".to_string(),
                    description: "GRUB has password protection (not managed by GRUBST).".to_string(),
                    severity: AuditSeverity::Pass,
                    fix_hint: None,
                };
            }
        }
    }

    AuditItem {
        title: "GRUB Password Protection".to_string(),
        description: "No password protection on GRUB. Anyone can edit boot parameters or access recovery mode.".to_string(),
        severity: AuditSeverity::Critical,
        fix_hint: Some("Use the Lock feature to enable GRUB password protection.".to_string()),
    }
}

fn check_grub_permissions() -> AuditItem {
    let grub_cfg = Path::new("/boot/grub/grub.cfg");

    if !grub_cfg.exists() {
        return AuditItem {
            title: "GRUB Config Permissions".to_string(),
            description: "grub.cfg not found at expected path.".to_string(),
            severity: AuditSeverity::Warning,
            fix_hint: Some("Verify GRUB is installed correctly.".to_string()),
        };
    }

    if let Ok(metadata) = fs::metadata(grub_cfg) {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();

        // grub.cfg should be readable only by root (mode 0400 or 0600)
        if mode & 0o077 == 0 {
            return AuditItem {
                title: "GRUB Config Permissions".to_string(),
                description: format!("grub.cfg permissions are restrictive (mode {:04o}).", mode & 0o777),
                severity: AuditSeverity::Pass,
                fix_hint: None,
            };
        } else {
            return AuditItem {
                title: "GRUB Config Permissions".to_string(),
                description: format!(
                    "grub.cfg is readable by non-root users (mode {:04o}). This may leak password hashes.",
                    mode & 0o777
                ),
                severity: AuditSeverity::Warning,
                fix_hint: Some("Run: sudo chmod 600 /boot/grub/grub.cfg".to_string()),
            };
        }
    }

    AuditItem {
        title: "GRUB Config Permissions".to_string(),
        description: "Could not read grub.cfg metadata.".to_string(),
        severity: AuditSeverity::Info,
        fix_hint: None,
    }
}

fn check_disk_encryption() -> AuditItem {
    // Check for LUKS encrypted partitions
    let output = Command::new("lsblk")
        .args(["-o", "NAME,TYPE", "-n", "-l"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("crypt") {
            return AuditItem {
                title: "Full Disk Encryption".to_string(),
                description: "LUKS encryption detected on at least one partition.".to_string(),
                severity: AuditSeverity::Pass,
                fix_hint: None,
            };
        }
    }

    // Also check /etc/crypttab
    if Path::new("/etc/crypttab").exists() {
        if let Ok(content) = fs::read_to_string("/etc/crypttab") {
            if !content.trim().is_empty()
                && content.lines().any(|l| !l.starts_with('#') && !l.trim().is_empty())
            {
                return AuditItem {
                    title: "Full Disk Encryption".to_string(),
                    description: "Disk encryption entries found in /etc/crypttab.".to_string(),
                    severity: AuditSeverity::Pass,
                    fix_hint: None,
                };
            }
        }
    }

    AuditItem {
        title: "Full Disk Encryption".to_string(),
        description: "No disk encryption detected. Physical access to the disk exposes all data.".to_string(),
        severity: AuditSeverity::Critical,
        fix_hint: Some("Consider enabling LUKS full-disk encryption for maximum security.".to_string()),
    }
}

fn check_secure_boot() -> AuditItem {
    // Check if Secure Boot is enabled
    let sb_path = Path::new("/sys/firmware/efi/efivars");

    if !sb_path.exists() {
        return AuditItem {
            title: "Secure Boot".to_string(),
            description: "System is booting in Legacy/BIOS mode (no EFI). Secure Boot is not available.".to_string(),
            severity: AuditSeverity::Info,
            fix_hint: Some("Consider switching to UEFI boot mode if your hardware supports it.".to_string()),
        };
    }

    // Try to read Secure Boot state
    let output = Command::new("mokutil")
        .arg("--sb-state")
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("SecureBoot enabled") {
            return AuditItem {
                title: "Secure Boot".to_string(),
                description: "Secure Boot is enabled. Only signed bootloaders can execute.".to_string(),
                severity: AuditSeverity::Pass,
                fix_hint: None,
            };
        } else if stdout.contains("SecureBoot disabled") {
            return AuditItem {
                title: "Secure Boot".to_string(),
                description: "Secure Boot is disabled. Unsigned bootloaders can execute.".to_string(),
                severity: AuditSeverity::Warning,
                fix_hint: Some("Enable Secure Boot in BIOS/UEFI settings for additional protection.".to_string()),
            };
        }
    }

    AuditItem {
        title: "Secure Boot".to_string(),
        description: "Could not determine Secure Boot status.".to_string(),
        severity: AuditSeverity::Info,
        fix_hint: Some("Install mokutil: sudo apt install mokutil".to_string()),
    }
}

fn check_boot_order() -> AuditItem {
    // Check if BIOS password is set (we can't directly check this)
    // We can check if efibootmgr shows USB as first boot device

    let output = Command::new("efibootmgr").output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lower = stdout.to_lowercase();

        // Check boot order — if USB/removable is before disk, it's a risk
        if lower.contains("usb")
            && lower.find("usb").unwrap_or(9999)
                < lower.find("ubuntu").or(lower.find("grub")).unwrap_or(9999)
        {
            return AuditItem {
                title: "Boot Order Security".to_string(),
                description: "USB boot has higher priority than the main OS. An attacker could boot from USB.".to_string(),
                severity: AuditSeverity::Warning,
                fix_hint: Some("Set the hard drive as the first boot device in BIOS/UEFI and set a BIOS password.".to_string()),
            };
        }

        return AuditItem {
            title: "Boot Order Security".to_string(),
            description: "UEFI boot order detected. Verify that BIOS password is set to prevent changes.".to_string(),
            severity: AuditSeverity::Info,
            fix_hint: Some("Set a BIOS/UEFI administrator password to prevent boot order changes.".to_string()),
        };
    }

    AuditItem {
        title: "Boot Order Security".to_string(),
        description: "Could not check boot order (efibootmgr not available or running in Legacy mode).".to_string(),
        severity: AuditSeverity::Info,
        fix_hint: Some("Set a BIOS password and disable USB boot from BIOS settings.".to_string()),
    }
}

fn check_root_password() -> AuditItem {
    // Check if root account has a password set
    if let Ok(shadow) = fs::read_to_string("/etc/shadow") {
        for line in shadow.lines() {
            if line.starts_with("root:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1 {
                    let hash = parts[1];
                    if hash == "*" || hash == "!" || hash == "!!" {
                        return AuditItem {
                            title: "Root Account".to_string(),
                            description: "Root account is locked (no direct root login). This is good practice.".to_string(),
                            severity: AuditSeverity::Pass,
                            fix_hint: None,
                        };
                    } else if hash.starts_with('$') {
                        return AuditItem {
                            title: "Root Account".to_string(),
                            description: "Root account has a password set. Direct root login is possible.".to_string(),
                            severity: AuditSeverity::Info,
                            fix_hint: Some("Consider locking root: sudo passwd -l root (use sudo instead).".to_string()),
                        };
                    }
                }
            }
        }
    }

    AuditItem {
        title: "Root Account".to_string(),
        description: "Could not check root account status (insufficient privileges).".to_string(),
        severity: AuditSeverity::Info,
        fix_hint: Some("Run GRUBST as root for a complete audit.".to_string()),
    }
}

fn check_update_guard() -> AuditItem {
    let guard_path = Path::new("/etc/grub.d/99_grubst_guard");

    if guard_path.exists() {
        AuditItem {
            title: "GRUBST Update Guard".to_string(),
            description: "Update guard is installed. GRUB protection survives system updates.".to_string(),
            severity: AuditSeverity::Pass,
            fix_hint: None,
        }
    } else {
        let superuser_script = Path::new("/etc/grub.d/01_grubst_users");
        if superuser_script.exists() {
            // Protection is on but guard is missing
            AuditItem {
                title: "GRUBST Update Guard".to_string(),
                description: "GRUBST protection is active but update guard is missing. Running 'update-grub' might remove protection.".to_string(),
                severity: AuditSeverity::Warning,
                fix_hint: Some("Re-run the Lock process to install the update guard.".to_string()),
            }
        } else {
            AuditItem {
                title: "GRUBST Update Guard".to_string(),
                description: "No GRUBST protection active. Update guard is not applicable.".to_string(),
                severity: AuditSeverity::Info,
                fix_hint: None,
            }
        }
    }
}

fn check_backup_status() -> AuditItem {
    let backups = crate::core::backup::list_backups();

    if backups.is_empty() {
        AuditItem {
            title: "GRUB Backup".to_string(),
            description: "No GRUBST backups found. If something goes wrong, manual recovery may be needed.".to_string(),
            severity: AuditSeverity::Info,
            fix_hint: Some("A backup is automatically created when you use the Lock feature.".to_string()),
        }
    } else {
        AuditItem {
            title: "GRUB Backup".to_string(),
            description: format!(
                "{} backup(s) available. Latest: {}",
                backups.len(),
                backups[0].created_at
            ),
            severity: AuditSeverity::Pass,
            fix_hint: None,
        }
    }
}
