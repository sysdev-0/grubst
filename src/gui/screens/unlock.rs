use dioxus::prelude::*;
use crate::core::grub::{check_protection_status, ProtectionStatus, unlock_grub};
use crate::gui::ActiveScreen;

#[component]
pub fn UnlockScreen(current_screen: Signal<ActiveScreen>, home_refresh_key: Signal<u64>) -> Element {
    let mut password = use_signal(|| "".to_string());
    let mut status = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut is_working = use_signal(|| false);
    let mut is_protected = use_signal(|| true);

    use_effect(move || {
        let protected = matches!(
            check_protection_status(),
            Ok(ProtectionStatus::Protected { .. })
        );
        is_protected.set(protected);
    });

    rsx! {
        div {
            h1 { class: "page-title", "Unlock GRUB" }
            p { class: "page-subtitle", "Remove bootloader protection and restore defaults" }

            if !is_protected() {
                div { class: "glass-panel fade-in",
                    div { class: "text-center", style: "padding: 40px 0;",
                        h2 { style: "font-size: 50px; margin-bottom: 16px;", "🔓" }
                        h3 { style: "margin-bottom: 12px;", "GRUB is Not Locked" }
                        p { style: "color: var(--text-muted); margin-bottom: 24px;",
                            "No active protection on the bootloader. Use the Lock feature to enable protection."
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| current_screen.set(ActiveScreen::Lock),
                            "🔒 Lock GRUB"
                        }
                    }
                }
            } else {
                div { class: "glass-panel fade-in",
                    div { style: "background: var(--warning-glow); border: 1px solid var(--warning); padding: 16px; border-radius: 8px; margin-bottom: 20px;",
                        h4 { style: "color: var(--warning); margin-bottom: 8px;", "⚠️ WARNING" }
                        p { style: "color: var(--text-main); font-size: 14px; line-height: 1.5;",
                            "Unlocking will remove all GRUB protection."
                            br {}
                            "Your bootloader will be restored to its default open state."
                        }
                    }

                    div { class: "input-group mt-20",
                        label { class: "input-label", "Backup Password" }
                        input {
                            class: "text-input",
                            r#type: "password",
                            placeholder: "Enter your backup password to confirm unlock",
                            value: "{password()}",
                            oninput: move |evt| password.set(evt.value())
                        }
                    }

                    div { class: "mt-20",
                        button {
                            class: "btn btn-danger",
                            disabled: password().is_empty(),
                            onclick: move |_| {
                                status.set(None);
                                error.set(None);
                                is_working.set(true);

                                let pwd = password();
                                let mut status = status;
                                let mut error = error;
                                let mut is_working = is_working;

                                spawn(async move {
                                    let result = tokio::task::spawn_blocking(move || unlock_grub(&pwd)).await;
                                    match result {
                                        Ok(Ok(())) => {
                                            is_working.set(false);
                                            status.set(Some("GRUB unlocked successfully.".to_string()));
                                        }
                                        Ok(Err(e)) => {
                                            is_working.set(false);
                                            error.set(Some(e));
                                        }
                                        Err(e) => {
                                            is_working.set(false);
                                            error.set(Some(format!("Task failed: {}", e)));
                                        }
                                    }
                                });
                            },
                            "🔓 Unlock GRUB"
                        }
                    }

                    if is_working() {
                        div { style: "margin-top: 16px; color: var(--text-muted);", "Working..." }
                    }

                    if let Some(msg) = status() {
                        div { style: "margin-top: 16px; background: rgba(34, 197, 94, 0.12); border: 1px solid rgba(34, 197, 94, 0.4); padding: 12px; border-radius: 8px;",
                            p { style: "color: var(--text-main);", "{msg}" }
                        }
                        div { class: "mt-20",
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    current_screen.set(ActiveScreen::Home);
                                    home_refresh_key.with_mut(|v| *v = v.wrapping_add(1));
                                },
                                "Return to Dashboard"
                            }
                        }
                    }

                    if let Some(err) = error() {
                        div { style: "margin-top: 16px; background: var(--danger-glow); border: 1px solid var(--danger); padding: 12px; border-radius: 8px;",
                            p { style: "color: var(--text-main); white-space: pre-wrap;", "{err}" }
                        }
                    }
                }
            }
        }
    }
}
