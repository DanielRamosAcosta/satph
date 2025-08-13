use crate::domain::config::{ConfigError, Environment, Settings};
use std::env;

#[derive(Debug, Default)]
pub struct SystemEnvironment;

impl Environment for SystemEnvironment {
    fn var(&self, key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_environment(&SystemEnvironment)
    }
}
