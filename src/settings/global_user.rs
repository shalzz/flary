use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use cloudflare::framework::auth::Credentials;
use serde::{Deserialize, Serialize};

use crate::settings::{Environment, QueryEnvironment};

const DEFAULT_CONFIG_FILE_NAME: &str = "default.toml";

const CF_API_TOKEN: &str = "CF_API_TOKEN";
const CF_API_KEY: &str = "CF_API_KEY";
const CF_EMAIL: &str = "CF_EMAIL";

static ENV_VAR_WHITELIST: [&str; 3] = [CF_API_TOKEN, CF_API_KEY, CF_EMAIL];

use anyhow::Result;
#[cfg(test)]
use std::io::Write;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum GlobalUser {
    TokenAuth { api_token: String },
    GlobalKeyAuth { email: String, api_key: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct WranglerConfig {
    pub oauth_token: Option<String>,
    pub api_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expiration_time: Option<String>,
    pub scopes: Option<Vec<String>>,
}

impl WranglerConfig {
    pub fn read_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string(self)?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn merge_scopes(&mut self, new_scopes: &[&str]) {
        let existing = self.scopes.get_or_insert_with(Vec::new);
        for scope in new_scopes {
            if !existing.iter().any(|s| s == scope) {
                existing.push(scope.to_string());
            }
        }
    }
}

impl GlobalUser {
    pub fn new() -> Result<Self> {
        let environment = Environment::with_whitelist(ENV_VAR_WHITELIST.to_vec());

        let config_path = get_global_config_path()?;
        GlobalUser::build(environment, config_path)
    }

    fn build<T: 'static + QueryEnvironment>(environment: T, config_path: PathBuf) -> Result<Self>
    where
        T: config::Source + Send + Sync,
    {
        if let Some(user) = Self::from_env(environment) {
            user
        } else if let Some(user) = Self::from_local_wrangler() {
            user
        } else {
            Self::from_file(config_path)
        }
    }

    fn from_env<T: 'static + QueryEnvironment>(environment: T) -> Option<Result<Self>>
    where
        T: config::Source + Send + Sync,
    {
        if environment.empty().unwrap_or(true) {
            None
        } else {
            let mut s = config::Config::new();
            s.merge(environment).ok();

            Some(GlobalUser::from_config(s))
        }
    }

    fn from_local_wrangler() -> Option<Result<Self>> {
        let local_config = Path::new("wrangler.toml");
        if !local_config.exists() {
            return None;
        }

        log::info!("Found local wrangler.toml, reading OAuth token");
        match Self::from_wrangler_file(local_config) {
            Ok(user) => Some(Ok(user)),
            Err(e) => {
                log::info!("Failed to read local wrangler.toml: {}", e);
                None
            }
        }
    }

    fn from_file(config_path: PathBuf) -> Result<Self> {
        if !config_path.exists() {
            anyhow::bail!(
                "config path does not exist {}. Try running `wrangler config`",
                config_path.display()
            );
        }

        log::info!(
            "Config path exists. Reading from config file, {}",
            config_path.display()
        );

        Self::from_wrangler_file(&config_path)
    }

    fn from_wrangler_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse_wrangler_content(&content)
    }

    fn parse_wrangler_content(content: &str) -> Result<Self> {
        let config: WranglerConfig = toml::from_str(content)
            .map_err(|e| anyhow::anyhow!("Failed to parse wrangler config: {}", e))?;

        // Prefer api_token (set via CLOUDFLARE_API_TOKEN or stored as such)
        if let Some(api_token) = config.api_token {
            if !api_token.trim().is_empty() {
                log::info!("Using api_token from wrangler config");
                return Ok(GlobalUser::TokenAuth { api_token });
            }
        }

        // Fall back to oauth_token (from wrangler login)
        if let Some(oauth_token) = config.oauth_token {
            if !oauth_token.trim().is_empty() {
                log::info!("Using oauth_token from wrangler config");
                return Ok(GlobalUser::TokenAuth {
                    api_token: oauth_token,
                });
            }
        }

        anyhow::bail!(
            "No valid token found in wrangler config. \
             Expected `api_token` or `oauth_token` field."
        )
    }

    pub fn to_file(&self, config_path: &Path) -> Result<()> {
        let toml = toml::to_string(self)?;

        fs::create_dir_all(&config_path.parent().unwrap())?;
        fs::write(&config_path, toml)?;

        Ok(())
    }

    fn from_config(config: config::Config) -> Result<Self> {
        let global_user: Result<GlobalUser, config::ConfigError> = config.clone().try_into();
        match global_user {
            Ok(user) => Ok(user),
            Err(_) => {
                let msg =
                    "Your authentication details are improperly configured.\nPlease run `wrangler config` or visit\nhttps://developers.cloudflare.com/workers/tooling/wrangler/configuration/#using-environment-variables\nfor info on configuring with environment variables";
                log::info!("{:?}", config);
                Err(anyhow!(msg))
            }
        }
    }
}

impl From<GlobalUser> for Credentials {
    fn from(user: GlobalUser) -> Credentials {
        match user {
            GlobalUser::TokenAuth { api_token } => Credentials::UserAuthToken { token: api_token },
            GlobalUser::GlobalKeyAuth { email, api_key } => Credentials::UserAuthKey {
                key: api_key,
                email,
            },
        }
    }
}

pub fn get_global_config_path() -> Result<PathBuf> {
    let home_dir = if let Ok(value) = env::var("WRANGLER_HOME") {
        log::info!("Using $WRANGLER_HOME: {}", value);
        Path::new(&value).to_path_buf()
    } else {
        log::info!("No $WRANGLER_HOME detected, using $HOME");
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".wrangler")
    };
    let global_config_file = home_dir.join("config").join(DEFAULT_CONFIG_FILE_NAME);
    log::info!("Using global config file: {}", global_config_file.display());
    Ok(global_config_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    use crate::settings::environment::MockEnvironment;

    #[test]
    fn it_can_prioritize_token_input() {
        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_API_TOKEN, "foo");
        mock_env.set(CF_EMAIL, "test@cloudflare.com");
        mock_env.set(CF_API_KEY, "bar");

        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let user = GlobalUser::build(mock_env, config_dir).unwrap();
        assert_eq!(
            user,
            GlobalUser::TokenAuth {
                api_token: "foo".to_string()
            }
        );
    }

    #[test]
    fn it_can_prioritize_env_vars() {
        let api_token = "thisisanapitoken";
        let api_key = "reallylongglobalapikey";
        let email = "user@example.com";

        let file_user = GlobalUser::TokenAuth {
            api_token: api_token.to_string(),
        };
        let env_user = GlobalUser::GlobalKeyAuth {
            api_key: api_key.to_string(),
            email: email.to_string(),
        };

        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_EMAIL, email);
        mock_env.set(CF_API_KEY, api_key);

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(file_user)).unwrap();

        let new_user = GlobalUser::build(mock_env, tmp_config_path).unwrap();

        assert_eq!(new_user, env_user);
    }

    #[test]
    fn it_falls_through_to_config_with_no_env_vars() {
        let mock_env = MockEnvironment::default();

        let user = GlobalUser::TokenAuth {
            api_token: "thisisanapitoken".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let tmp_config_path = test_config_dir(&tmp_dir, Some(user.clone())).unwrap();

        let new_user = GlobalUser::build(mock_env, tmp_config_path).unwrap();

        assert_eq!(new_user, user);
    }

    #[test]
    fn it_fails_if_no_token_in_file() {
        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&config_dir.as_path())
            .unwrap();
        let toml_content = "refresh_token = \"some-refresh-token\"\nexpiration_time = \"2099-01-01T00:00:00Z\"\nscopes = []\n";
        file.write_all(toml_content.as_bytes()).unwrap();

        let file_user = GlobalUser::from_file(config_dir);

        assert!(file_user.is_err());
    }

    #[test]
    fn it_fails_if_global_auth_incomplete_in_env() {
        let mut mock_env = MockEnvironment::default();

        mock_env.set(CF_API_KEY, "apikey");

        let tmp_dir = tempdir().unwrap();
        let config_dir = test_config_dir(&tmp_dir, None).unwrap();

        let new_user = GlobalUser::build(mock_env, config_dir);

        assert!(new_user.is_err());
    }

    #[test]
    fn it_succeeds_with_no_config() {
        let mut mock_env = MockEnvironment::default();
        mock_env.set(CF_API_KEY, "apikey");
        mock_env.set(CF_EMAIL, "email");
        let dummy_path = Path::new("./definitely-does-not-exist.txt").to_path_buf();
        let new_user = GlobalUser::build(mock_env, dummy_path);

        assert!(new_user.is_ok());
    }

    #[test]
    fn it_parses_oauth_token_from_wrangler_config() {
        let content = r#"
oauth_token = "my-oauth-token-123"
refresh_token = "some-refresh-token"
expiration_time = "2099-01-01T00:00:00Z"
scopes = ["account:read"]
"#;

        let user = GlobalUser::parse_wrangler_content(content).unwrap();
        assert_eq!(
            user,
            GlobalUser::TokenAuth {
                api_token: "my-oauth-token-123".to_string()
            }
        );
    }

    #[test]
    fn it_prefers_api_token_over_oauth_token() {
        let content = r#"
api_token = "my-api-token-456"
oauth_token = "my-oauth-token-123"
"#;

        let user = GlobalUser::parse_wrangler_content(content).unwrap();
        assert_eq!(
            user,
            GlobalUser::TokenAuth {
                api_token: "my-api-token-456".to_string()
            }
        );
    }

    #[test]
    fn it_converts_token_auth_to_credentials() {
        let user = GlobalUser::TokenAuth {
            api_token: "test-token".to_string(),
        };
        let creds: Credentials = user.into();
        assert_eq!(
            creds.headers(),
            vec![("Authorization", "Bearer test-token".to_string())]
        );
    }

    fn test_config_dir(tmp_dir: &tempfile::TempDir, user: Option<GlobalUser>) -> Result<PathBuf> {
        let tmp_config_path = tmp_dir.path().join(DEFAULT_CONFIG_FILE_NAME);
        if let Some(user_config) = user {
            user_config.to_file(&tmp_config_path)?;
        } else {
            File::create(&tmp_config_path)?;
        }

        Ok(tmp_config_path)
    }

    #[test]
    fn it_converts_global_key_auth_to_credentials() {
        let user = GlobalUser::GlobalKeyAuth {
            email: "user@example.com".to_string(),
            api_key: "my-api-key".to_string(),
        };
        let creds: Credentials = user.into();
        match creds {
            Credentials::UserAuthKey { key, email } => {
                assert_eq!(key, "my-api-key");
                assert_eq!(email, "user@example.com");
            }
            _ => panic!("Expected UserAuthKey"),
        }
    }

    #[test]
    fn it_roundtrips_to_file_and_back() {
        let user = GlobalUser::TokenAuth {
            api_token: "roundtrip-token".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("test.toml");

        user.to_file(&config_path).unwrap();
        let loaded = GlobalUser::from_file(config_path).unwrap();

        assert_eq!(user, loaded);
    }

    #[test]
    fn it_roundtrips_global_key_auth_to_file_matches_format() {
        let user = GlobalUser::GlobalKeyAuth {
            email: "test@example.com".to_string(),
            api_key: "test-key-123".to_string(),
        };

        let tmp_dir = tempdir().unwrap();
        let config_path = tmp_dir.path().join("test.toml");

        user.to_file(&config_path).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test@example.com"));
        assert!(content.contains("test-key-123"));
    }
}
