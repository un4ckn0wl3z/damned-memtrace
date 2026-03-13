//! Memory Traversal Tool
//! A desktop application for memory pointer traversal
#![windows_subsystem = "windows"]

use std::env;
use dioxus::desktop::{LogicalSize, WindowBuilder};
use ui::App;

fn main() {
    let user_data_dir = env::var("LOCALAPPDATA").expect("env var LOCALAPPDATA not found");
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_disable_context_menu(true)
                .with_data_directory(user_data_dir)
                .with_window(
                    WindowBuilder::new()
                        .with_title("Damned Memory Traversal Tool")
                        .with_decorations(false)
                        .with_inner_size(LogicalSize::new(1100.0, 700.0))
                        .with_resizable(true),
                ),
        )
        .launch(App);
}
