// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::{AppLauncher, PlatformError, WindowDesc};

mod config;
mod layout;
mod loader;
mod modal_host;
mod updater;

fn main() -> Result<(), PlatformError> {
    match config::init() {
        Ok(_) => run(),
        Err(e) => run_without_config(e),
    }
}

fn run() -> Result<(), PlatformError> {
    let data = loader::load_data().unwrap();
    let state = layout::AppState::build(data);
    let main_window = WindowDesc::new(layout::build_ui).title("Traduora-Update");
    AppLauncher::with_window(main_window)
        .delegate(layout::Delegate)
        .use_simple_logger()
        .launch(state)
}

fn run_without_config(err: anyhow::Error) -> Result<(), PlatformError> {
    let window = WindowDesc::new(layout::build_ui_load_config_failed).title("Traduora-Update");
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(err.into())
}
