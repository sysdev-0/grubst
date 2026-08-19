pub mod screens;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveScreen {
    Home,
    Lock,
    Unlock,
    Audit,
}

#[component]
pub fn App() -> Element {
    let mut current_screen = use_signal(|| ActiveScreen::Home);
    let mut home_refresh_key = use_signal(|| 0u64);
    
    // Check if root (just a visual warning if not root)
    let is_root = use_signal(|| {
        #[cfg(target_os = "linux")]
        {
            nix::unistd::geteuid().is_root()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    });

    let home_class = if current_screen() == ActiveScreen::Home { "nav-item active" } else { "nav-item" };
    let lock_class = if current_screen() == ActiveScreen::Lock { "nav-item active" } else { "nav-item" };
    let unlock_class = if current_screen() == ActiveScreen::Unlock { "nav-item active" } else { "nav-item" };
    let audit_class = if current_screen() == ActiveScreen::Audit { "nav-item active" } else { "nav-item" };

    rsx! {
        div { id: "app",
            // Sidebar
            div { class: "sidebar",
                div { class: "sidebar-header",
                    span { style: "font-size: 28px;", "🛡️" }
                    h1 { "GRUBST" }
                }
                
                div {
                    class: "{home_class}",
                    onclick: move |_| {
                        current_screen.set(ActiveScreen::Home);
                        home_refresh_key.with_mut(|v| *v = v.wrapping_add(1));
                    },
                    "🏠 Dashboard"
                }
                div {
                    class: "{lock_class}",
                    onclick: move |_| current_screen.set(ActiveScreen::Lock),
                    "🔒 Lock GRUB"
                }
                div {
                    class: "{unlock_class}",
                    onclick: move |_| current_screen.set(ActiveScreen::Unlock),
                    "🔓 Unlock"
                }
                div {
                    class: "{audit_class}",
                    onclick: move |_| current_screen.set(ActiveScreen::Audit),
                    "🔍 Security Audit"
                }
                
                div { class: "sidebar-footer",
                    "GRUBST v2"
                    br {}
                    "Boot Security Hardening"
                }
            }

            // Main Content Area
            div { class: "content fade-in",
                if !is_root() {
                    div {
                        style: "background: var(--danger-glow); border: 1px solid var(--danger); padding: 12px; border-radius: 8px; margin-bottom: 20px; color: white;",
                        "⚠️ Warning: You are not running as root. Most features require 'sudo'."
                    }
                }
                
                {
                    match current_screen() {
                        ActiveScreen::Home => rsx! {
                            screens::home::Home {
                                current_screen: current_screen,
                                home_refresh_key: home_refresh_key,
                            }
                        },
                        ActiveScreen::Lock => rsx! { screens::lock::LockWizard { current_screen: current_screen, home_refresh_key: home_refresh_key } },
                        ActiveScreen::Unlock => rsx! { screens::unlock::UnlockScreen { current_screen: current_screen, home_refresh_key: home_refresh_key } },
                        ActiveScreen::Audit => rsx! { screens::audit::AuditScreen {} },
                    }
                }
            }
        }
    }
}
