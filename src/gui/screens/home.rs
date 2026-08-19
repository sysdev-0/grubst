use dioxus::prelude::*;
use crate::core::grub::check_protection_status;
use crate::core::{backup, update_guard};
use crate::gui::ActiveScreen;

#[component]
pub fn Home(current_screen: Signal<ActiveScreen>, home_refresh_key: Signal<u64>) -> Element {
    let mut is_protected = use_signal(|| false);
    let mut details = use_signal(|| "Checking status...".to_string());
    let mut guard_details = use_signal(|| "Guard: Unknown".to_string());
    let mut backup_details = use_signal(|| "Last backup: Unknown".to_string());

    // Run this effect once when the component mounts
    use_effect(move || {
        let _ = home_refresh_key();
        match check_protection_status() {
            Ok(crate::core::grub::ProtectionStatus::Protected { locked_at, usb_uuid }) => {
                is_protected.set(true);
                let mut msg = format!("Locked at: {}\nUSB UUID: {}", locked_at, usb_uuid);
                if let Ok(content) = std::fs::read_to_string("/var/lib/grubst/state.json") {
                    if let Ok(state) = serde_json::from_str::<serde_json::Value>(&content) {
                        if state
                            .get("unrestricted_in_10_linux")
                            .and_then(|v| v.as_bool())
                            == Some(false)
                        {
                            msg.push_str("\nWarning: normal boot may require password on this distro.");
                        }
                    }
                }
                details.set(msg);
            }
            Ok(crate::core::grub::ProtectionStatus::Unprotected) => {
                is_protected.set(false);
                details.set("System is currently unprotected.".to_string());
            }
            Err(e) => {
                is_protected.set(false);
                details.set(format!("Error: {}", e));
            }
        }

        let guard_installed = update_guard::is_guard_installed();
        guard_details.set(if guard_installed {
            "Guard: Installed".to_string()
        } else {
            "Guard: Not installed".to_string()
        });

        let backups = backup::list_backups();
        backup_details.set(if let Some(latest) = backups.first() {
            format!("Last backup: {}", latest.created_at)
        } else {
            "Last backup: None".to_string()
        });
    });

    let os_name = sysinfo::System::name().unwrap_or_else(|| "Linux".to_string());
    let cpu_arch = std::env::consts::ARCH;

    let panel_class = if is_protected() { "glass-panel glowing" } else { "glass-panel" };
    let icon_class = if is_protected() { "status-icon protected" } else { "status-icon unprotected" };
    let status_icon_text = if is_protected() { "🔒" } else { "🔓" };
    let status_title = if is_protected() { "GRUB is Protected" } else { "GRUB is Unprotected" };

    rsx! {
        div {
            h1 { class: "page-title", "Dashboard" }
            p { class: "page-subtitle", "Overview of your bootloader security status" }
            
            div { class: "status-grid",
                // Main Status Card
                div { class: "{panel_class}",
                    div { class: "status-card",
                        div { class: "{icon_class}",
                            "{status_icon_text}"
                        }
                        div { class: "status-info",
                            h3 {
                                "{status_title}"
                            }
                            p { "{details()}" }
                        }
                    }
                }
                
                // System Info Card
                div { class: "glass-panel",
                    div { class: "status-card",
                        div { class: "status-icon", style: "background: rgba(59, 130, 246, 0.1); color: var(--accent-blue); border: 1px solid var(--border-dim);",
                            "💻"
                        }
                        div { class: "status-info",
                            h3 { "System Info" }
                            p { "OS: {os_name}" }
                            p { "Platform: {cpu_arch}" }
                            p { "{guard_details()}" }
                            p { "{backup_details()}" }
                        }
                    }
                }
            }
            
            div { class: "glass-panel",
                h3 { style: "margin-bottom: 12px;", "Quick Actions" }
                div { class: "flex-row",
                    if !is_protected() {
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| current_screen.set(ActiveScreen::Lock),
                            "🔒 Secure GRUB Now"
                        }
                    } else {
                        button {
                            class: "btn btn-danger",
                            onclick: move |_| current_screen.set(ActiveScreen::Unlock),
                            "🔓 Unlock GRUB"
                        }
                    }
                    button {
                        class: "btn",
                        onclick: move |_| current_screen.set(ActiveScreen::Audit),
                        "🔍 Run Security Audit"
                    }
                }
            }
        }
    }
}
