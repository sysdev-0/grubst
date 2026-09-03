use dioxus::prelude::*;
use crate::core::audit::{run_audit, AuditReport};

#[component]
pub fn AuditScreen() -> Element {
    let mut report = use_signal(|| Option::<AuditReport>::None);

    rsx! {
        div {
            div { class: "flex-row space-between align-center",
                div {
                    h1 { class: "page-title", "Security Audit" }
                    p { class: "page-subtitle", "Comprehensive security analysis of your boot configuration" }
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        report.set(Some(run_audit()));
                    },
                    "↻ Run Scan"
                }
            }
            
            if let Some(rpt) = report() {
                {
                    let score = rpt.score;
                    let color = if score >= 80 { "var(--success)" } else if score >= 50 { "var(--warning)" } else { "var(--danger)" };
                    let glow = if score >= 80 { "var(--success-glow)" } else if score >= 50 { "var(--warning-glow)" } else { "var(--danger-glow)" };
                    let border = format!("4px solid {}", color);
                    let shadow = format!("0 0 20px {}", glow);
                    
                    let critical_count = rpt.items.iter().filter(|i| i.severity == crate::core::audit::AuditSeverity::Critical).count();
                    let warning_count = rpt.items.iter().filter(|i| i.severity == crate::core::audit::AuditSeverity::Warning).count();
                    let pass_count = rpt.items.iter().filter(|i| i.severity == crate::core::audit::AuditSeverity::Pass).count();
                    let sum = rpt.summary.clone();
                    
                    rsx! {
                        div { class: "fade-in",
                            // Score Card
                            div { class: "glass-panel flex-row align-center",
                                div { 
                                    style: "width: 80px; height: 80px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 32px; font-weight: bold; border: {border}; color: {color}; background: rgba(0,0,0,0.3); box-shadow: {shadow};",
                                    "{score}"
                                }
                                div { style: "margin-left: 20px;",
                                    h3 { style: "font-size: 20px; margin-bottom: 4px;", "Security Score" }
                                    p { style: "color: var(--text-muted); margin-bottom: 8px;", "{sum}" }
                                    div { class: "flex-row", style: "font-size: 12px;",
                                        span { style: "color: var(--danger);", "⛔ {critical_count} Critical" }
                                        span { style: "color: var(--warning);", "⚠️ {warning_count} Warnings" }
                                        span { style: "color: var(--success);", "✅ {pass_count} Passed" }
                                    }
                                }
                            }
                            
                            // Audit Items
                            div { class: "mt-20",
                                for item in rpt.items.iter() {
                                    {
                                        let class_name = match item.severity {
                                            crate::core::audit::AuditSeverity::Critical => "audit-item critical",
                                            crate::core::audit::AuditSeverity::Warning => "audit-item warning",
                                            crate::core::audit::AuditSeverity::Info => "audit-item info",
                                            crate::core::audit::AuditSeverity::Pass => "audit-item pass",
                                        };
                                        let icon = match item.severity {
                                            crate::core::audit::AuditSeverity::Critical => "⛔",
                                            crate::core::audit::AuditSeverity::Warning => "⚠️",
                                            crate::core::audit::AuditSeverity::Info => "ℹ️",
                                            crate::core::audit::AuditSeverity::Pass => "✅",
                                        };
                                        let item_title = item.title.clone();
                                        let item_desc = item.description.clone();
                                        let item_fix = item.fix_hint.clone();
                                        
                                        rsx! {
                                            div {
                                                class: "{class_name}",
                                                div { style: "font-size: 20px; padding-top: 2px;",
                                                    "{icon}"
                                                }
                                                div { class: "audit-content",
                                                    h4 { "{item_title}" }
                                                    p { "{item_desc}" }
                                                    if let Some(fix) = item_fix {
                                                        div { class: "audit-hint",
                                                            "💡 {fix}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "text-center", style: "margin-top: 100px;",
                    div { style: "font-size: 60px; margin-bottom: 20px; animation: pulse-glow 2s infinite alternate;", "🛡️" }
                    h3 { style: "color: var(--text-muted);", "Click 'Run Scan' to audit your bootloader security" }
                }
            }
        }
    }
}
