// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::{AppLauncher, PlatformError, WindowDesc};

mod config;
mod layout;
mod loader;
mod modal_host;
mod updater;

fn main() -> Result<(), PlatformError> {
    let data = loader::load_data().unwrap();
    let state = layout::AppState::build(data);
    let main_window = WindowDesc::new(layout::build_ui);
    AppLauncher::with_window(main_window)
        .delegate(layout::Delegate)
        .use_simple_logger()
        .launch(state)
}
