use dioxus::prelude::*;
use crate::core::grub::lock_grub;
use crate::core::usb::{detect_usb_devices, UsbDevice};
use crate::gui::ActiveScreen;


#[derive(Clone, PartialEq, Debug)]
enum LockStep {
    SelectUsb,
    ConfirmFormat,
    BackupPassword,
    Processing,
    Complete,
}

#[component]
pub fn LockWizard(current_screen: Signal<ActiveScreen>, home_refresh_key: Signal<u64>) -> Element {
    let mut current_step = use_signal(|| LockStep::SelectUsb);
    let mut devices = use_signal(|| Vec::<UsbDevice>::new());
    let mut selected_device = use_signal(|| Option::<UsbDevice>::None);
    let mut backup_password = use_signal(|| "".to_string());
    let mut format_confirm = use_signal(|| "".to_string());
    
    // Processing state
    let _progress = use_signal(|| 0.0);
    let mut progress_msg = use_signal(|| "Preparing...".to_string());
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut is_working = use_signal(|| false);

    use_effect(move || {
        // Load devices on mount
        devices.set(detect_usb_devices());
    });

    rsx! {
        div {
            h1 { class: "page-title", "Lock GRUB" }
            p { class: "page-subtitle", "Secure your bootloader with a physical USB key" }
            
            // Wizard Steps indicator
            div { class: "wizard-steps",
                WizardStepIndicator { 
                    step_num: 1, 
                    label: "Select USB".to_string(), 
                    active: current_step() == LockStep::SelectUsb, 
                    completed: current_step() != LockStep::SelectUsb && current_step() != LockStep::ConfirmFormat 
                }
                WizardStepIndicator { 
                    step_num: 2, 
                    label: "Format".to_string(), 
                    active: current_step() == LockStep::ConfirmFormat, 
                    completed: current_step() == LockStep::BackupPassword || current_step() == LockStep::Processing || current_step() == LockStep::Complete 
                }
                WizardStepIndicator { 
                    step_num: 3, 
                    label: "Password".to_string(), 
                    active: current_step() == LockStep::BackupPassword, 
                    completed: current_step() == LockStep::Processing || current_step() == LockStep::Complete 
                }
                WizardStepIndicator { 
                    step_num: 4, 
                    label: "Apply".to_string(), 
                    active: current_step() == LockStep::Processing, 
                    completed: current_step() == LockStep::Complete 
                }
            }
            
            // Step Content
            div { class: "glass-panel fade-in",
                {
                    match current_step() {
                        LockStep::SelectUsb => rsx! {
                            h3 { style: "margin-bottom: 16px;", "Step 1: Select Rescue Key" }
                            p { class: "mb-20", style: "color: var(--text-muted);", 
                                "Insert a USB drive to act as your physical unlock key. Minimum 4GB required." 
                            }
                            
                            if devices().is_empty() {
                                div { style: "text-align: center; padding: 30px; background: rgba(0,0,0,0.2); border-radius: 8px;",
                                    h2 { "📭 No USB devices found" }
                                    p { style: "color: var(--text-muted); margin-top: 10px;", "Please insert a USB drive and click refresh." }
                                }
                            } else {
                                div {
                                    for dev in devices().iter() {
                                        {
                                            let is_selected = selected_device().as_ref().map(|d| &d.path) == Some(&dev.path);
                                            let dev_class = if is_selected { "device-card selected" } else { "device-card" };
                                            let d = dev.clone();
                                            let title = dev.name.clone();
                                            let sub = format!("{} — {}", dev.path, dev.size_human);
                                            
                                            rsx! {
                                                div {
                                                    class: "{dev_class}",
                                                    onclick: move |_| selected_device.set(Some(d.clone())),
                                                    div {
                                                        h4 { "{title}" }
                                                        p { style: "font-size: 12px; color: var(--text-muted);", "{sub}" }
                                                    }
                                                    div { "💾" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            div { class: "flex-row space-between mt-20",
                                button { 
                                    class: "btn", 
                                    onclick: move |_| devices.set(detect_usb_devices()),
                                    "🔄 Refresh Devices" 
                                }
                                button {
                                    class: "btn btn-primary",
                                    disabled: selected_device().is_none(),
                                    onclick: move |_| current_step.set(LockStep::ConfirmFormat),
                                    "Next ➔"
                                }
                            }
                        },
                        
                        LockStep::ConfirmFormat => rsx! {
                            h3 { style: "margin-bottom: 16px; color: var(--danger);", "Step 2: Confirm Format" }
                            div { style: "background: var(--danger-glow); border: 1px solid var(--danger); padding: 16px; border-radius: 8px; margin-bottom: 20px;",
                                h4 { style: "color: white; margin-bottom: 8px;", "⚠️ WARNING: DATA LOSS" }
                                p { style: "color: var(--text-main); font-size: 14px; line-height: 1.5;",
                                    "The selected USB drive will be completely erased and formatted."
                                    br {}
                                    "All existing files will be permanently deleted."
                                }
                            }

                            if let Some(dev) = selected_device() {
                                div { style: "background: rgba(0,0,0,0.2); border: 1px solid var(--border-dim); padding: 12px; border-radius: 8px; margin-bottom: 16px;",
                                    p { style: "color: var(--text-muted); font-size: 12px; margin-bottom: 6px;", "Selected device" }
                                    p { style: "font-size: 14px;", "{dev.name} — {dev.path} — {dev.size_human}" }
                                }
                            }

                            div { class: "input-group",
                                label { class: "input-label", "Type the device path to confirm" }
                                input {
                                    class: "text-input",
                                    placeholder: "/dev/sdX",
                                    value: "{format_confirm()}",
                                    oninput: move |evt| format_confirm.set(evt.value())
                                }
                            }
                            
                            div { class: "flex-row space-between mt-20",
                                button { 
                                    class: "btn", 
                                    onclick: move |_| current_step.set(LockStep::SelectUsb),
                                    "🡠 Back" 
                                }
                                button {
                                    class: "btn btn-danger",
                                    disabled: selected_device()
                                        .as_ref()
                                        .map(|d| format_confirm().trim() != d.path.as_str())
                                        .unwrap_or(true),
                                    onclick: move |_| current_step.set(LockStep::BackupPassword),
                                    "Yes, Format USB"
                                }
                            }
                        },
                        
                        LockStep::BackupPassword => rsx! {
                            h3 { style: "margin-bottom: 16px;", "Step 3: Backup Password" }
                            p { class: "mb-20", style: "color: var(--text-muted);", 
                                "Create a backup password in case you lose the physical USB key. Store it securely." 
                            }
                            
                            div { class: "input-group",
                                label { class: "input-label", "Password" }
                                input {
                                    class: "text-input",
                                    r#type: "password",
                                    placeholder: "Enter a strong password",
                                    value: "{backup_password()}",
                                    oninput: move |evt| backup_password.set(evt.value())
                                }
                            }
                            
                            div { class: "flex-row space-between mt-20",
                                button { 
                                    class: "btn", 
                                    onclick: move |_| current_step.set(LockStep::ConfirmFormat),
                                    "🡠 Back" 
                                }
                                button {
                                    class: "btn btn-primary",
                                    disabled: backup_password().len() < 8,
                                    onclick: move |_| {
                                        error_msg.set(None);
                                        current_step.set(LockStep::Processing);
                                        progress_msg.set("Applying protection...".to_string());
                                        is_working.set(true);

                                        let dev = selected_device().unwrap();
                                        let pwd = backup_password();
                                        let mut current_step = current_step;
                                        let mut progress_msg = progress_msg;
                                        let mut error_msg = error_msg;
                                        let mut is_working = is_working;

                                        spawn(async move {
                                            let result = tokio::task::spawn_blocking(move || lock_grub(&dev, &pwd)).await;
                                            match result {
                                                Ok(Ok(())) => {
                                                    is_working.set(false);
                                                    progress_msg.set("Protection applied".to_string());
                                                    current_step.set(LockStep::Complete);
                                                }
                                                Ok(Err(e)) => {
                                                    is_working.set(false);
                                                    error_msg.set(Some(e));
                                                    progress_msg.set("Failed".to_string());
                                                }
                                                Err(e) => {
                                                    is_working.set(false);
                                                    error_msg.set(Some(format!("Task failed: {}", e)));
                                                    progress_msg.set("Failed".to_string());
                                                }
                                            }
                                        });
                                    },
                                    "Lock GRUB ➔"
                                }
                            }
                        },
                        
                        LockStep::Processing => rsx! {
                            h3 { style: "margin-bottom: 16px;", "Step 4: Applying Protection" }
                            div { class: "text-center", style: "margin: 40px 0;",
                                h2 { style: "font-size: 40px; margin-bottom: 20px;", "⚙️" }
                                h4 { style: "color: var(--glow-cyan); margin-bottom: 10px;", "{progress_msg()}" }
                                div { class: "progress-container",
                                    div { class: "progress-bar indeterminate" } 
                                }
                            }

                            if let Some(err) = error_msg() {
                                div { style: "background: var(--danger-glow); border: 1px solid var(--danger); padding: 12px; border-radius: 8px; margin-bottom: 16px;",
                                    h4 { style: "color: white; margin-bottom: 6px;", "Failed to lock GRUB" }
                                    p { style: "color: var(--text-main); font-size: 13px; line-height: 1.5; white-space: pre-wrap;", "{err}" }
                                }
                            }

                            div { class: "flex-row space-between",
                                button {
                                    class: "btn",
                                    disabled: is_working(),
                                    onclick: move |_| current_step.set(LockStep::BackupPassword),
                                    "🡠 Back"
                                }
                                button {
                                    class: "btn btn-primary",
                                    disabled: is_working() || error_msg().is_some(),
                                    onclick: move |_| current_step.set(LockStep::Complete),
                                    "Continue"
                                }
                            }
                        },
                        
                        LockStep::Complete => rsx! {
                            div { class: "text-center", style: "padding: 30px 0;",
                                h2 { style: "font-size: 60px; margin-bottom: 20px;", "✅" }
                                h2 { style: "color: var(--success); margin-bottom: 10px;", "GRUB Successfully Locked!" }
                                p { style: "color: var(--text-muted); margin-bottom: 30px;", 
                                    "Your bootloader is now protected. Keep your USB rescue key safe." 
                                }
                                button {
                                    class: "btn btn-primary",
                                    onclick: move |_| {
                                        current_step.set(LockStep::SelectUsb);
                                        selected_device.set(None);
                                        backup_password.set("".to_string());
                                        format_confirm.set("".to_string());
                                        progress_msg.set("Preparing...".to_string());
                                        error_msg.set(None);
                                        is_working.set(false);
                                        current_screen.set(ActiveScreen::Home);
                                        home_refresh_key.with_mut(|v| *v = v.wrapping_add(1));
                                    },
                                    "Return to Dashboard"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WizardStepIndicator(step_num: u32, label: String, active: bool, completed: bool) -> Element {
    let classes = if active { "step active" } else if completed { "step completed" } else { "step" };
    
    rsx! {
        div { class: "{classes}",
            div { class: "step-circle", 
                if completed { "✓" } else { "{step_num}" } 
            }
            div { class: "step-label", "{label}" }
        }
    }
}
