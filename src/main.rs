// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

use anyhow::{Context, Result};
use druid::{AppLauncher, PlatformError, WindowDesc};

mod config;
mod layout;
mod loader;
mod modal_host;
mod updater;

fn main() -> Result<()> {
    if write_schema()? {
        return Ok(());
    }

    let config_result = config::init();
    match config_result.and_then(|_| loader::load_data()) {
        Ok(data) => run(data),
        Err(e) => run_startup_failed(e),
    }
    .map_err(Into::into)
}

fn write_schema() -> Result<bool> {
    use itertools::Itertools;

    std::env::args_os()
        .tuple_windows()
        .find_map(|(pred, succ)| (pred == "--generate-config-schema").then(|| succ))
        .map_or(Ok(false), |schema_file| {
            let schema = schemars::schema_for!(config::AppConfig);
            let schema =
                serde_json::to_string_pretty(&schema).context("Failed to generate schema.")?;
            std::fs::write(schema_file, schema).context("Failed to save schema to file.")?;
            Ok(true)
        })
}

fn run(data: Vec<loader::Translation>) -> Result<(), PlatformError> {
    let state = layout::AppState::build(data);
    let main_window = WindowDesc::new(layout::build_ui).title("Traduora-Update");
    AppLauncher::with_window(main_window)
        .delegate(layout::Delegate)
        .use_simple_logger()
        .launch(state)
}

fn run_startup_failed(err: anyhow::Error) -> Result<(), PlatformError> {
    let window = WindowDesc::new(layout::build_ui_startup_failed).title("Traduora-Update");
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(err.into())
}
