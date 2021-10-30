mod data;
mod local;
mod remote;

use data::Translation;

use local::load_from_file;
use remote::fetch_from_traduora;

pub use data::build_app_state;
