#![allow(unused)]
// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use druid::{AppLauncher, PlatformError, WindowDesc};

mod layout;
mod load;

fn main() -> Result<(), PlatformError> {
    let data = load::build_app_state().unwrap();
    let main_window = WindowDesc::new(layout::build_ui);
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}
