use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use schemars::JsonSchema;
use serde::Deserialize;
use traduora::{
    api::{locales::LocaleCode, ProjectId},
    auth::Authenticated,
    Login, Traduora, TraduoraBuilder,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum LoginConfig {
    Password {
        /// Normal user account for Traduora login
        #[schemars(email)]
        mail: String,
        /// User password for Traduora login
        password: String,
    },
    ClientCredentials {
        /// Id of a Traduora API client for login
        client_id: String,
        /// Secret of a Traduora API client for login
        client_secret: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Encoding {
    Combined {
        #[serde(deserialize_with = "de_helper::deserialize_encoding")]
        #[schemars(
            with = "String",
            example = "de_helper::example::encoding_utf_8",
            example = "de_helper::example::encoding_utf_16"
        )]
        // asdasd
        git: &'static encoding_rs::Encoding,
        #[serde(deserialize_with = "de_helper::deserialize_encoding")]
        #[schemars(
            with = "String",
            example = "de_helper::example::encoding_utf_8",
            example = "de_helper::example::encoding_utf_16"
        )]
        // asdasd
        local: &'static encoding_rs::Encoding,
    },
    GitOnly {
        #[serde(deserialize_with = "de_helper::deserialize_encoding")]
        #[schemars(
            with = "String",
            example = "de_helper::example::encoding_utf_8",
            example = "de_helper::example::encoding_utf_16"
        )]
        // asdasd
        git: &'static encoding_rs::Encoding,
    },
    LocalOnly {
        #[serde(deserialize_with = "de_helper::deserialize_encoding")]
        #[schemars(
            with = "String",
            example = "de_helper::example::encoding_utf_8",
            example = "de_helper::example::encoding_utf_16"
        )]
        // asdasd
        local: &'static encoding_rs::Encoding,
    },
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AppConfig {
    #[serde(flatten)]
    login: LoginConfig,
    /// URL to access the Traduora instance
    #[schemars(url)]
    host: String,
    /// Locale that should be updated
    #[schemars(
        with = "String",
        example = "de_helper::example::locale_en",
        example = "de_helper::example::locale_de_de",
        example = "de_helper::example::locale_ru"
    )]
    locale: LocaleCode,
    /// Path to file that contains the translations. Should be formatted like JSON-flat
    /// export of Traduora. Relative path from working directory.
    translation_file: PathBuf,
    /// Id of the project that should be updated
    #[schemars(with = "String", example = "de_helper::example::project_id")]
    project_id: ProjectId,
    /// Whether the connection to the server should be encrypted. Defaults to true.
    #[schemars(default = "de_helper::bool_true")]
    with_ssl: bool,
    /// Whether the encryption certificates should be validated. Defaults to true.
    #[schemars(default = "de_helper::bool_true")]
    validate_certs: bool,
    /// Git revision to use for sanity checks to prevent changing terms by mistake.
    /// Can be any valid revision, e.g. commit hash, tag, branch. Should usually be
    /// your default branch. If omitted, sanity checks are skipped.
    #[serde(default)]
    #[schemars(
        example = "de_helper::example::revision_branch",
        example = "de_helper::example::revision_tag",
        example = "de_helper::example::revision_commit"
    )]
    revision: String,
    /// Encoding of the translation file. Used for both the local version and the git version.
    /// If omitted, the tool tries to determine the encoding automatically via its byte order mark
    /// or just assumes UTF-8 on failure.
    #[serde(default)]
    #[schemars(skip_serializing)]
    encoding: Option<Encoding>,
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

    /// Get a reference to the app config's revision.
    pub fn revision(&self) -> &str {
        self.revision.as_ref()
    }

    /// Get a reference to the app config's git encoding.
    pub fn encoding_git(&self) -> Option<&'static encoding_rs::Encoding> {
        match self.encoding.as_ref()? {
            Encoding::Combined { git, .. } => Some(git),
            Encoding::GitOnly { git } => Some(git),
            Encoding::LocalOnly { .. } => None,
        }
    }

    /// Get a reference to the app config's local file encoding.
    pub fn encoding_local(&self) -> Option<&'static encoding_rs::Encoding> {
        match self.encoding.as_ref()? {
            Encoding::Combined { local, .. } => Some(local),
            Encoding::LocalOnly { local } => Some(local),
            Encoding::GitOnly { .. } => None,
        }
    }
}

mod de_helper {
    use std::result::Result;

    use encoding_rs::Encoding;
    use serde::Deserializer;

    pub fn bool_true() -> bool {
        true
    }

    pub mod example {
        pub fn project_id() -> &'static str {
            "92047938-c050-4d9c-83f8-6b1d7fae6b01"
        }

        pub fn locale_en() -> &'static str {
            "en"
        }

        pub fn locale_de_de() -> &'static str {
            "de_DE"
        }

        pub fn locale_ru() -> &'static str {
            "ru"
        }

        pub fn revision_commit() -> &'static str {
            "9011cdcd095d156c6a7e34182fdcba144ab1789a"
        }

        pub fn revision_branch() -> &'static str {
            "main"
        }

        pub fn revision_tag() -> &'static str {
            "v2.7.41"
        }

        pub fn encoding_utf_8() -> &'static str {
            "utf-8"
        }

        pub fn encoding_utf_16() -> &'static str {
            "utf-16"
        }
    }

    pub fn deserialize_encoding<'de, D>(de: D) -> Result<&'static Encoding, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};
        struct Helper;

        impl<'de> Visitor<'de> for Helper {
            type Value = &'static Encoding;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an encoding")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Encoding::for_label(value.as_bytes())
                    .ok_or_else(|| Error::custom("Failed to parse encoding."))
            }
        }

        de.deserialize_str(Helper)
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

    let config = parse(config_file)?;

    CONFIG
        .set(config)
        .expect("Configuration was already loaded.");

    Ok(())
}

fn parse(config_file: impl AsRef<Path>) -> Result<AppConfig> {
    use json_comments::StripComments;

    let jsonc = std::fs::read_to_string(&config_file)
        .with_context(|| format!("Failed to read config file {:?}", config_file.as_ref()))?;

    let json = StripComments::new(jsonc.as_bytes());

    serde_json::from_reader(json)
        .with_context(|| format!("Failed to parse config file {:?}", config_file.as_ref()))
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

#[cfg(test)]
pub fn init_test() {
    let _ = CONFIG.set(AppConfig {
        login: LoginConfig::Password {
            mail: "test@test.test".into(),
            password: "12345678".into(),
        },
        host: "localhost:8080".into(),
        locale: "en".into(),
        translation_file: "testdata/en.json".into(),
        project_id: "92047938-c050-4d9c-83f8-6b1d7fae6b01".into(),
        with_ssl: false,
        validate_certs: false,
        revision: String::new(),
        encoding: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config() {
        let config = parse("./traduora-update.json").unwrap();
        assert_eq!(
            Encoding::Combined {
                git: encoding_rs::UTF_8,
                local: encoding_rs::UTF_16LE
            },
            config.encoding.unwrap()
        );
    }

    #[test]
    fn schema() {
        let schema = schemars::schema_for!(AppConfig);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
