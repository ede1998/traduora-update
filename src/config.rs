use anyhow::{Context, Result};
use traduora::{auth::Authenticated, Login, Traduora};

pub const USER: &str = "test@test.test";
pub const PWD: &str = "12345678";
pub const HOST: &str = "localhost:8080";
pub const LOCALE: &str = "en";
pub const PROJECT_ID: &str = "92047938-c050-4d9c-83f8-6b1d7fae6b01";

pub fn create_client() -> Result<Traduora<Authenticated>> {
    Traduora::with_auth_insecure(HOST, Login::password(USER, PWD)).with_context(|| {
        format!(
            "Login failed for Traduora instance {:?} (user: {:?})",
            HOST, USER
        )
    })
}
