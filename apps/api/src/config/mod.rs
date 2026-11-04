//! Application configuration.
//!
//! Every value is read from the environment exactly once at startup, validated,
//! and then frozen inside [`AppConfig`]. No other module reads `std::env`
//! directly, which keeps configuration traceable.

use std::{collections::HashMap, env, fmt, str::FromStr, time::Duration};

use thiserror::Error;

use crate::utils::redact::redact_url;

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_LOG_LEVEL: &str = "info";
const DEFAULT_HEALTH_CHECK_TIMEOUT_MS: u64 = 2_000;
const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const DEFAULT_MIN_CONNECTIONS: u32 = 1;
const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 5;
const DEFAULT_PUBLIC_RATE_LIMIT_PER_MINUTE: u32 = 120;
const DEFAULT_SOURCE_SYNC_INTERVAL_SECS: u64 = 300;
const DEFAULT_SOURCE_HTTP_TIMEOUT_MS: u64 = 15_000;
const DEFAULT_SOURCE_STALE_SECS: u64 = 900;

/// Runtime environment of the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Test,
    Production,
}

impl AppEnv {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Test => "test",
            Self::Production => "production",
        }
    }

    pub const fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AppEnv {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "test" | "testing" => Ok(Self::Test),
            "production" | "prod" => Ok(Self::Production),
            other => Err(ConfigError::Invalid {
                key: "APP_ENV",
                reason: format!("expected development|test|production, got `{other}`"),
            }),
        }
    }
}

/// Errors raised while loading configuration. Messages never contain secrets.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable `{key}`")]
    Missing { key: &'static str },
    #[error("invalid value for environment variable `{key}`: {reason}")]
    Invalid { key: &'static str, reason: String },
}

/// Validated, immutable application configuration.
#[derive(Clone)]
pub struct AppConfig {
    pub env: AppEnv,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub log_level: String,
    pub cors_allowed_origins: Vec<String>,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_acquire_timeout: Duration,
    pub redis_url: String,
    pub redis_key_prefix: String,
    pub health_check_timeout: Duration,
    /// Maximum requests per client IP per minute for `/api/v1/public/*`.
    pub public_rate_limit_per_minute: u32,
    /// Optional token for protected cache maintenance endpoints.
    pub maintenance_token: Option<String>,
    pub kzgo_base_url: String,
    pub kzgo_sync_enabled: bool,
    pub gokz_global_base_url: String,
    pub gokz_global_sync_enabled: bool,
    pub source_sync_interval: Duration,
    pub source_http_timeout: Duration,
    /// External snapshot age after which public reads fall back to plugin data.
    pub source_fallback_max_age: Duration,
}

impl AppConfig {
    /// Load configuration from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(env::vars())
    }

    /// Load configuration from an arbitrary key/value source.
    ///
    /// Kept separate from [`Self::from_env`] so tests do not have to mutate
    /// process wide state.
    pub fn from_pairs<I, K, V>(pairs: I) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let vars: HashMap<String, String> = pairs
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();

        let get = |key: &str| vars.get(key).map(|value| value.trim().to_owned());

        let env = match get("APP_ENV") {
            Some(value) if !value.is_empty() => value.parse()?,
            _ => AppEnv::Development,
        };

        let host = get("APP_HOST")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_HOST.to_owned());

        let port = parse_u16(&vars, "APP_PORT", DEFAULT_PORT)?;
        let version = get("APP_VERSION")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned());

        let log_level = get("LOG_LEVEL")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_owned());

        let cors_allowed_origins = parse_origins(&vars)?;

        let database_url = require(&vars, "DATABASE_URL")?;
        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err(ConfigError::Invalid {
                key: "DATABASE_URL",
                reason: "expected a postgres:// or postgresql:// URL".to_owned(),
            });
        }

        let database_max_connections = parse_u32(
            &vars,
            "DATABASE_MAX_CONNECTIONS",
            DEFAULT_MAX_CONNECTIONS,
            1,
            100,
        )?;
        let database_min_connections = parse_u32(
            &vars,
            "DATABASE_MIN_CONNECTIONS",
            DEFAULT_MIN_CONNECTIONS,
            0,
            100,
        )?;
        if database_min_connections > database_max_connections {
            return Err(ConfigError::Invalid {
                key: "DATABASE_MIN_CONNECTIONS",
                reason: "must be lower than or equal to DATABASE_MAX_CONNECTIONS".to_owned(),
            });
        }

        let acquire_secs = parse_u64(
            &vars,
            "DATABASE_ACQUIRE_TIMEOUT_SECS",
            DEFAULT_ACQUIRE_TIMEOUT_SECS,
            1,
            300,
        )?;

        let redis_url = require(&vars, "REDIS_URL")?;
        if !redis_url.starts_with("redis://") && !redis_url.starts_with("rediss://") {
            return Err(ConfigError::Invalid {
                key: "REDIS_URL",
                reason: "expected a redis:// or rediss:// URL".to_owned(),
            });
        }
        let redis_key_prefix = get("REDIS_KEY_PREFIX")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "cngokz".to_owned());

        let public_rate_limit_per_minute = parse_u32(
            &vars,
            "PUBLIC_RATE_LIMIT_PER_MINUTE",
            DEFAULT_PUBLIC_RATE_LIMIT_PER_MINUTE,
            1,
            10_000,
        )?;
        let maintenance_token = get("MAINTENANCE_TOKEN").filter(|value| !value.is_empty());
        let kzgo_base_url = optional_url(&vars, "KZGO_BASE_URL", "https://kzgo.eu")?;
        let gokz_global_base_url = optional_url(
            &vars,
            "GOKZ_GLOBAL_BASE_URL",
            "https://kztimerglobal.com/api/v2.0",
        )?;
        let kzgo_sync_enabled = parse_bool(&vars, "KZGO_SYNC_ENABLED", true)?;
        let gokz_global_sync_enabled = parse_bool(&vars, "GOKZ_GLOBAL_SYNC_ENABLED", false)?;
        let source_sync_interval_secs = parse_u64(
            &vars,
            "SOURCE_SYNC_INTERVAL_SECS",
            DEFAULT_SOURCE_SYNC_INTERVAL_SECS,
            30,
            86_400,
        )?;
        let source_http_timeout_ms = parse_u64(
            &vars,
            "SOURCE_HTTP_TIMEOUT_MS",
            DEFAULT_SOURCE_HTTP_TIMEOUT_MS,
            1_000,
            120_000,
        )?;
        let source_stale_secs = parse_u64(
            &vars,
            "SOURCE_FALLBACK_MAX_AGE_SECS",
            DEFAULT_SOURCE_STALE_SECS,
            60,
            86_400,
        )?;
        let health_check_timeout_ms = parse_u64(
            &vars,
            "HEALTH_CHECK_TIMEOUT_MS",
            DEFAULT_HEALTH_CHECK_TIMEOUT_MS,
            100,
            30_000,
        )?;

        Ok(Self {
            env,
            host,
            port,
            version,
            log_level,
            cors_allowed_origins,
            database_url,
            database_max_connections,
            database_min_connections,
            database_acquire_timeout: Duration::from_secs(acquire_secs),
            redis_url,
            redis_key_prefix,
            health_check_timeout: Duration::from_millis(health_check_timeout_ms),
            public_rate_limit_per_minute,
            maintenance_token,
            kzgo_base_url,
            kzgo_sync_enabled,
            gokz_global_base_url,
            gokz_global_sync_enabled,
            source_sync_interval: Duration::from_secs(source_sync_interval_secs),
            source_http_timeout: Duration::from_millis(source_http_timeout_ms),
            source_fallback_max_age: Duration::from_secs(source_stale_secs),
        })
    }

    pub fn is_cors_wildcard(&self) -> bool {
        self.cors_allowed_origins.iter().any(|origin| origin == "*")
    }

    pub fn socket_addr(&self) -> Result<std::net::SocketAddr, ConfigError> {
        let raw = format!("{}:{}", self.host, self.port);
        raw.parse().map_err(|error| ConfigError::Invalid {
            key: "APP_HOST",
            reason: format!("`{raw}` is not a valid socket address ({error})"),
        })
    }
}

/// Manual `Debug` implementation: URLs may contain credentials, so they are
/// redacted before they can end up in logs.
impl fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppConfig")
            .field("env", &self.env)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("version", &self.version)
            .field("log_level", &self.log_level)
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("database_url", &redact_url(&self.database_url))
            .field("database_max_connections", &self.database_max_connections)
            .field("database_min_connections", &self.database_min_connections)
            .field("database_acquire_timeout", &self.database_acquire_timeout)
            .field("redis_url", &redact_url(&self.redis_url))
            .field("redis_key_prefix", &self.redis_key_prefix)
            .field("health_check_timeout", &self.health_check_timeout)
            .field(
                "public_rate_limit_per_minute",
                &self.public_rate_limit_per_minute,
            )
            .field(
                "maintenance_token",
                &self.maintenance_token.as_ref().map(|_| "***"),
            )
            .field("kzgo_base_url", &redact_url(&self.kzgo_base_url))
            .field("kzgo_sync_enabled", &self.kzgo_sync_enabled)
            .field(
                "gokz_global_base_url",
                &redact_url(&self.gokz_global_base_url),
            )
            .field("gokz_global_sync_enabled", &self.gokz_global_sync_enabled)
            .field("source_sync_interval", &self.source_sync_interval)
            .field("source_http_timeout", &self.source_http_timeout)
            .field("source_fallback_max_age", &self.source_fallback_max_age)
            .finish()
    }
}

fn optional_url(
    vars: &HashMap<String, String>,
    key: &'static str,
    default: &str,
) -> Result<String, ConfigError> {
    let value = vars
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(default);
    if !value.starts_with("http://") && !value.starts_with("https://") {
        return Err(ConfigError::Invalid {
            key,
            reason: "expected an http:// or https:// URL".to_owned(),
        });
    }
    Ok(value.trim_end_matches('/').to_owned())
}

fn parse_bool(
    vars: &HashMap<String, String>,
    key: &'static str,
    default: bool,
) -> Result<bool, ConfigError> {
    match vars
        .get(key)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
    {
        Some(value) => match value.as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(ConfigError::Invalid {
                key,
                reason: format!("expected true/false, got `{value}`"),
            }),
        },
        None => Ok(default),
    }
}

fn require(vars: &HashMap<String, String>, key: &'static str) -> Result<String, ConfigError> {
    match vars
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(value) => Ok(value.to_owned()),
        None => Err(ConfigError::Missing { key }),
    }
}

fn parse_origins(vars: &HashMap<String, String>) -> Result<Vec<String>, ConfigError> {
    let raw = vars
        .get("CORS_ALLOWED_ORIGINS")
        .map(String::as_str)
        .unwrap_or("");
    let origins: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    for origin in &origins {
        let valid =
            origin == "*" || origin.starts_with("http://") || origin.starts_with("https://");
        if !valid {
            return Err(ConfigError::Invalid {
                key: "CORS_ALLOWED_ORIGINS",
                reason: format!("`{origin}` must be `*` or start with http:// or https://"),
            });
        }
    }

    Ok(origins)
}

fn parse_u16(
    vars: &HashMap<String, String>,
    key: &'static str,
    default: u16,
) -> Result<u16, ConfigError> {
    match vars
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(value) => value.parse().map_err(|_| ConfigError::Invalid {
            key,
            reason: format!(
                "expected a number between 0 and {}, got `{value}`",
                u16::MAX
            ),
        }),
        None => Ok(default),
    }
}

fn parse_u32(
    vars: &HashMap<String, String>,
    key: &'static str,
    default: u32,
    min: u32,
    max: u32,
) -> Result<u32, ConfigError> {
    match vars
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(value) => {
            let parsed: u32 = value.parse().map_err(|_| ConfigError::Invalid {
                key,
                reason: format!("expected a number between {min} and {max}, got `{value}`"),
            })?;
            if !(min..=max).contains(&parsed) {
                return Err(ConfigError::Invalid {
                    key,
                    reason: format!("must be between {min} and {max}, got `{parsed}`"),
                });
            }
            Ok(parsed)
        }
        None => Ok(default),
    }
}

fn parse_u64(
    vars: &HashMap<String, String>,
    key: &'static str,
    default: u64,
    min: u64,
    max: u64,
) -> Result<u64, ConfigError> {
    match vars
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(value) => {
            let parsed: u64 = value.parse().map_err(|_| ConfigError::Invalid {
                key,
                reason: format!("expected a number between {min} and {max}, got `{value}`"),
            })?;
            if !(min..=max).contains(&parsed) {
                return Err(ConfigError::Invalid {
                    key,
                    reason: format!("must be between {min} and {max}, got `{parsed}`"),
                });
            }
            Ok(parsed)
        }
        None => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_pairs() -> Vec<(&'static str, String)> {
        vec![
            ("APP_ENV", "development".to_owned()),
            (
                "DATABASE_URL",
                "postgres://user:secret@localhost:5432/db".to_owned(),
            ),
            ("REDIS_URL", "redis://localhost:6379".to_owned()),
            (
                "CORS_ALLOWED_ORIGINS",
                "http://localhost:5173, http://127.0.0.1:5173".to_owned(),
            ),
        ]
    }

    #[test]
    fn applies_defaults() {
        let config = AppConfig::from_pairs(base_pairs()).expect("config should load");

        assert_eq!(config.env, AppEnv::Development);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(config.log_level, "info");
        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.database_min_connections, 1);
        assert_eq!(config.database_acquire_timeout, Duration::from_secs(5));
        assert_eq!(config.health_check_timeout, Duration::from_millis(2_000));
        assert_eq!(config.public_rate_limit_per_minute, 120);
        assert_eq!(config.cors_allowed_origins.len(), 2);
    }

    #[test]
    fn fails_when_database_url_is_missing() {
        let pairs = base_pairs()
            .into_iter()
            .filter(|(key, _)| *key != "DATABASE_URL");
        let error = AppConfig::from_pairs(pairs).expect_err("config must fail");

        assert!(matches!(
            error,
            ConfigError::Missing {
                key: "DATABASE_URL"
            }
        ));
    }

    #[test]
    fn rejects_non_postgres_url() {
        let mut pairs = base_pairs();
        pairs.push(("DATABASE_URL", String::new()));
        pairs.retain(|(key, _)| *key != "DATABASE_URL");
        pairs.push(("DATABASE_URL", "mysql://localhost/db".to_owned()));

        let error = AppConfig::from_pairs(pairs).expect_err("config must fail");
        assert!(matches!(
            error,
            ConfigError::Invalid {
                key: "DATABASE_URL",
                ..
            }
        ));
    }

    #[test]
    fn rejects_unknown_environment() {
        let mut pairs = base_pairs();
        pairs.push(("APP_ENV", "staging".to_owned()));

        let error = AppConfig::from_pairs(pairs).expect_err("config must fail");
        assert!(matches!(error, ConfigError::Invalid { key: "APP_ENV", .. }));
    }

    #[test]
    fn rejects_invalid_cors_origin() {
        let mut pairs = base_pairs();
        pairs.push(("CORS_ALLOWED_ORIGINS", "localhost:5173".to_owned()));

        let error = AppConfig::from_pairs(pairs).expect_err("config must fail");
        assert!(matches!(
            error,
            ConfigError::Invalid {
                key: "CORS_ALLOWED_ORIGINS",
                ..
            }
        ));
    }

    #[test]
    fn rejects_min_connections_above_max() {
        let mut pairs = base_pairs();
        pairs.push(("DATABASE_MAX_CONNECTIONS", "2".to_owned()));
        pairs.push(("DATABASE_MIN_CONNECTIONS", "5".to_owned()));

        let error = AppConfig::from_pairs(pairs).expect_err("config must fail");
        assert!(matches!(
            error,
            ConfigError::Invalid {
                key: "DATABASE_MIN_CONNECTIONS",
                ..
            }
        ));
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let config = AppConfig::from_pairs(base_pairs()).expect("config should load");
        let debug = format!("{config:?}");

        assert!(
            !debug.contains("secret"),
            "debug output leaked the password: {debug}"
        );
        assert!(debug.contains("postgres://user:***@localhost:5432/db"));
    }
}
