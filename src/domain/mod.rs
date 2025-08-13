pub mod auth_service;
pub mod authelia;
pub mod config;
pub mod password;
pub mod username;

pub use authelia::Authelia;
pub use password::Password;
pub use username::Username;

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum Protocol {
    SSH,
    FTP,
    DAV,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::SSH => write!(f, "SSH"),
            Protocol::FTP => write!(f, "FTP"),
            Protocol::DAV => write!(f, "DAV"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub username: Username,
    pub password: Password,
    pub ip: std::net::IpAddr,
    pub protocol: Protocol,
}

#[derive(Debug, Clone)]
pub struct SessionCookie {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthResult {
    pub success: bool,
}

impl AuthResult {
    pub fn success() -> Self {
        Self { success: true }
    }

    #[allow(dead_code)]
    pub fn failure() -> Self {
        Self { success: false }
    }
}

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("Invalid input: {message}")]
    InputInvalid { message: String },

    #[error("First factor authentication failed")]
    FirstFactorUnauthorized,

    #[error("Second factor authentication failed")]
    SecondFactorUnauthorized,

    #[error("Upstream service unavailable")]
    UpstreamUnavailable,

    #[error("Request timeout")]
    Timeout,

    #[error("Unexpected error: {message}")]
    Unexpected { message: String },
}

impl AuthError {
    pub fn input_invalid<S: Into<String>>(message: S) -> Self {
        Self::InputInvalid {
            message: message.into(),
        }
    }

    pub fn unexpected<S: Into<String>>(message: S) -> Self {
        Self::Unexpected {
            message: message.into(),
        }
    }
}
