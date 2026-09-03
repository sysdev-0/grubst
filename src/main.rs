use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use grubst::gui::App;

fn main() {
    // Initialize logging
    env_logger::init();

    // Include the custom CSS as a string
    let css = include_str!("gui/style.css");
    
    // Configure the desktop window
    let config = Config::new()
        .with_custom_head(format!("<style>{}</style>", css))
        .with_window(
            WindowBuilder::new()
                .with_title("GRUBST — Boot Security")
                .with_inner_size(LogicalSize::new(1000.0, 700.0))
                .with_min_inner_size(LogicalSize::new(800.0, 600.0))
        );

    // Launch the Dioxus app
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
