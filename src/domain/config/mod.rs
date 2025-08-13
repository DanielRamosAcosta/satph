use std::env;
use std::time::Duration;
use thiserror::Error;

pub trait Environment {
    fn var(&self, key: &str) -> Result<String, env::VarError>;
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub authelia_base_url: String,
    pub http_bind: String,
    pub http_client_timeout: Duration,
    pub log_level: String,
    pub tls_insecure: bool,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {name}")]
    MissingEnvVar { name: String },

    #[error("Invalid value for {name}: {error}")]
    InvalidValue { name: String, error: String },
}

impl Settings {
    pub fn from_environment(env: &dyn Environment) -> Result<Self, ConfigError> {
        let authelia_base_url =
            env.var("AUTHELIA_BASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar {
                    name: "AUTHELIA_BASE_URL".to_string(),
                })?;

        url::Url::parse(&authelia_base_url).map_err(|_| ConfigError::InvalidValue {
            name: "AUTHELIA_BASE_URL".to_string(),
            error: "Invalid URL format".to_string(),
        })?;

        let http_bind = env
            .var("HTTP_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

        let http_client_timeout_ms = env
            .var("HTTP_CLIENT_TIMEOUT_MS")
            .unwrap_or_else(|_| "5000".to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::InvalidValue {
                name: "HTTP_CLIENT_TIMEOUT_MS".to_string(),
                error: format!("Must be a valid number: {}", e),
            })?;

        let log_level = env.var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let tls_insecure = env
            .var("TLS_INSECURE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .map_err(|e| ConfigError::InvalidValue {
                name: "TLS_INSECURE".to_string(),
                error: format!("Must be true or false: {}", e),
            })?;

        if tls_insecure {
            tracing::warn!("TLS_INSECURE is enabled - this is not safe for production use");
        }

        Ok(Settings {
            authelia_base_url,
            http_bind,
            http_client_timeout: Duration::from_millis(http_client_timeout_ms),
            log_level,
            tls_insecure,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.http_client_timeout.as_millis() == 0 {
            return Err(ConfigError::InvalidValue {
                name: "HTTP_CLIENT_TIMEOUT_MS".to_string(),
                error: "Timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    struct MockEnvironment {
        vars: HashMap<String, String>,
    }

    impl MockEnvironment {
        fn new() -> Self {
            Self {
                vars: HashMap::new(),
            }
        }

        fn set_var(mut self, key: &str, value: &str) -> Self {
            self.vars.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl Environment for MockEnvironment {
        fn var(&self, key: &str) -> Result<String, env::VarError> {
            self.vars.get(key).cloned().ok_or(env::VarError::NotPresent)
        }
    }

    #[test]
    fn test_settings_with_mock_environment_defaults() {
        let mock_env =
            MockEnvironment::new().set_var("AUTHELIA_BASE_URL", "https://authelia.test.com");

        let settings = Settings::from_environment(&mock_env).unwrap();

        assert_eq!(settings.authelia_base_url, "https://authelia.test.com");
        assert_eq!(settings.http_bind, "0.0.0.0:8080");
        assert_eq!(settings.http_client_timeout, Duration::from_millis(5000));
        assert_eq!(settings.log_level, "info");
        assert!(!settings.tls_insecure);
    }

    #[test]
    fn test_settings_with_mock_environment_custom_values() {
        let mock_env = MockEnvironment::new()
            .set_var("AUTHELIA_BASE_URL", "https://auth.test.com")
            .set_var("HTTP_BIND", "127.0.0.1:9090")
            .set_var("HTTP_CLIENT_TIMEOUT_MS", "10000")
            .set_var("LOG_LEVEL", "debug")
            .set_var("TLS_INSECURE", "true");

        let settings = Settings::from_environment(&mock_env).unwrap();

        assert_eq!(settings.authelia_base_url, "https://auth.test.com");
        assert_eq!(settings.http_bind, "127.0.0.1:9090");
        assert_eq!(settings.http_client_timeout, Duration::from_millis(10000));
        assert_eq!(settings.log_level, "debug");
        assert!(settings.tls_insecure);
    }

    #[test]
    fn test_settings_missing_required_var() {
        let mock_env = MockEnvironment::new();

        let result = Settings::from_environment(&mock_env);

        assert!(matches!(result, Err(ConfigError::MissingEnvVar { .. })));
        if let Err(ConfigError::MissingEnvVar { name }) = result {
            assert_eq!(name, "AUTHELIA_BASE_URL");
        }
    }

    #[test]
    fn test_settings_invalid_url() {
        let mock_env = MockEnvironment::new().set_var("AUTHELIA_BASE_URL", "not-a-valid-url");

        let result = Settings::from_environment(&mock_env);

        assert!(matches!(result, Err(ConfigError::InvalidValue { .. })));
    }

    #[test]
    fn test_settings_invalid_timeout() {
        let mock_env = MockEnvironment::new()
            .set_var("AUTHELIA_BASE_URL", "https://authelia.test.com")
            .set_var("HTTP_CLIENT_TIMEOUT_MS", "not-a-number");

        let result = Settings::from_environment(&mock_env);

        assert!(matches!(result, Err(ConfigError::InvalidValue { .. })));
    }

    #[test]
    fn test_settings_invalid_tls_insecure() {
        let mock_env = MockEnvironment::new()
            .set_var("AUTHELIA_BASE_URL", "https://authelia.test.com")
            .set_var("TLS_INSECURE", "maybe");

        let result = Settings::from_environment(&mock_env);

        assert!(matches!(result, Err(ConfigError::InvalidValue { .. })));
    }
}
