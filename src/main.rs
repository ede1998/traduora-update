// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::{AppLauncher, PlatformError, WindowDesc};

mod config;
mod layout;
mod loader;
mod updater;

fn main() -> Result<(), PlatformError> {
    let data = loader::load_data().unwrap();
    let state = layout::build_app_state(data);
    let main_window = WindowDesc::new(layout::build_ui);
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(state)
}
