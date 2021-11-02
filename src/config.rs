use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use traduora::{
    api::{locales::LocaleCode, ProjectId},
    auth::Authenticated,
    Login, Traduora, TraduoraBuilder,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LoginConfig {
    Password {
        mail: String,
        password: String,
    },
    ClientCredentials {
        client_id: String,
        client_secret: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(flatten)]
    login: LoginConfig,
    host: String,
    locale: LocaleCode,
    translation_file: PathBuf,
    project_id: ProjectId,
    with_ssl: bool,
    validate_certs: bool,
}

impl AppConfig {
    /// Get a reference to the app config's project id.
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    /// Get a reference to the app config's locale.
    pub fn locale(&self) -> &LocaleCode {
        &self.locale
    }

    /// Get a reference to the app config's host.
    pub fn host(&self) -> &str {
        self.host.as_ref()
    }

    /// Get a reference to the app config's login.
    pub fn login(&self) -> &LoginConfig {
        &self.login
    }

    /// Get a reference to the app config's translation file.
    pub fn translation_file(&self) -> &Path {
        &self.translation_file
    }

    /// Get a reference to the app config's with ssl.
    pub fn with_ssl(&self) -> bool {
        self.with_ssl
    }

    /// Get a reference to the app config's validate certs.
    pub fn validate_certs(&self) -> bool {
        self.validate_certs
    }
}

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

pub fn get() -> &'static AppConfig {
    CONFIG.get().expect("Configuration was not initialized")
}

pub fn init() -> Result<()> {
    let config_file = from_args()
        .or_else(from_env)
        .or_else(from_ascend_directories)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to find config file. Tried: \n
                1. reading command line argument\n
                2. reading environment variable TRADUORA_UPDATE_CONFIG,\n
                3. ascending directory tree and looking for traduora-update.json"
            )
        })?;
    let config = std::fs::read_to_string(&config_file)
        .with_context(|| format!("Failed to read config file {:?}", config_file))?;
    let config = serde_json::from_str(&config)
        .with_context(|| format!("Failed to parse config file {:?}", config_file))?;
    CONFIG
        .set(config)
        .expect("Configuration was already loaded.");

    Ok(())
}

fn from_args() -> Option<PathBuf> {
    std::env::args_os().nth(1).map(Into::into)
}

fn from_env() -> Option<PathBuf> {
    std::env::vars_os()
        .find_map(|(key, value)| (key == "TRADUORA_UPDATE_CONFIG").then(|| PathBuf::from(value)))
}

fn from_ascend_directories() -> Option<PathBuf> {
    match std::env::current_dir() {
        Ok(cwd) => cwd
            .ancestors()
            .find_map(|dir| match dir.read_dir() {
                Ok(mut entries) => entries.find_map(|entry| match entry {
                    Ok(f)
                        if f.file_name() == "traduora-update.json"
                            && File::open(f.path()).is_ok() =>
                    {
                        Some(f)
                    }
                    Ok(_) => None,
                    Err(e) => {
                        log::error!(
                            "Failed to read entry in directory {}. Silently ignoring it. Error: {}",
                            dir.display(),
                            e
                        );
                        None
                    }
                }),
                Err(e) => {
                    log::error!(
                        "Failed to read contents of directory {}. Silently ignoring it. Error: {}",
                        dir.display(),
                        e
                    );
                    None
                }
            })
            .map(|r| r.path()),
        Err(e) => {
            log::error!(
                "Failed to get current working directory. Silently ignoring it. Error: {}",
                e
            );
            None
        }
    }
}

pub fn create_client() -> Result<Traduora<Authenticated>> {
    let config = get();

    let (user, login) = match config.login() {
        LoginConfig::Password { mail, password } => (mail, Login::password(mail, password)),
        LoginConfig::ClientCredentials {
            client_id,
            client_secret,
        } => (
            client_id,
            Login::client_credentials(client_id, client_secret),
        ),
    };

    TraduoraBuilder::new(config.host())
        .authenticate(login)
        .use_http(!config.with_ssl())
        .validate_certs(config.validate_certs())
        .build()
        .with_context(|| {
            format!(
                "Login failed for Traduora instance {:?} (mail/client_id: {:?})",
                config.host(),
                user
            )
        })
}
