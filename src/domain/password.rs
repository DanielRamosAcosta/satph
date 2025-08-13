use crate::domain::AuthError;
use regex::Regex;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password {
    value: String,
}

impl Password {
    pub fn new(value: String) -> Result<Self, AuthError> {
        Self::validate(&value)?;
        Ok(Self { value })
    }

    fn validate(value: &str) -> Result<(), AuthError> {
        if value.len() < 7 {
            return Err(AuthError::input_invalid(
                "Password must be at least 7 characters (6 for TOTP + 1 for password)",
            ));
        }

        let password_len = value.len();
        let totp_token = &value[password_len - 6..];

        let totp_regex = Regex::new(r"^\d{6}$").unwrap();
        if !totp_regex.is_match(totp_token) {
            return Err(AuthError::input_invalid(
                "Last 6 characters must be numeric (TOTP token)",
            ));
        }

        Ok(())
    }

    pub fn first_factor_password(&self) -> String {
        let password_len = self.value.len();
        self.value[..password_len - 6].to_string()
    }

    pub fn totp_token(&self) -> String {
        let password_len = self.value.len();
        self.value[password_len - 6..].to_string()
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl From<Password> for String {
    fn from(password: Password) -> Self {
        password.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_valid() {
        let password = Password::new("password123456".to_string()).unwrap();
        assert_eq!(password.as_str(), "password123456");
    }

    #[test]
    fn test_password_valid_minimum_length() {
        let password = Password::new("a123456".to_string()).unwrap();
        assert_eq!(password.as_str(), "a123456");
        assert_eq!(password.first_factor_password(), "a");
        assert_eq!(password.totp_token(), "123456");
    }

    #[test]
    fn test_password_too_short() {
        let result = Password::new("short".to_string());
        assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
        if let Err(AuthError::InputInvalid { message }) = result {
            assert_eq!(message, "Password must be at least 7 characters (6 for TOTP + 1 for password)");
        }
    }

    #[test]
    fn test_password_invalid_totp() {
        let result = Password::new("passwordabcdef".to_string());
        assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
        if let Err(AuthError::InputInvalid { message }) = result {
            assert_eq!(message, "Last 6 characters must be numeric (TOTP token)");
        }
    }

    #[test]
    fn test_password_split_methods() {
        let password = Password::new("myPassword123456".to_string()).unwrap();
        assert_eq!(password.first_factor_password(), "myPassword");
        assert_eq!(password.totp_token(), "123456");
    }

    #[test]
    fn test_password_empty() {
        let result = Password::new("".to_string());
        assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
    }

    #[test]
    fn test_password_display_redacted() {
        let password = Password::new("password123456".to_string()).unwrap();
        assert_eq!(format!("{}", password), "[REDACTED]");
    }

    #[test]
    fn test_password_as_ref() {
        let password = Password::new("password123456".to_string()).unwrap();
        let as_ref: &str = password.as_ref();
        assert_eq!(as_ref, "password123456");
    }

    #[test]
    fn test_password_into_string() {
        let password = Password::new("password123456".to_string()).unwrap();
        let string_value: String = password.into();
        assert_eq!(string_value, "password123456");
    }

    #[test]
    fn test_password_equality() {
        let password1 = Password::new("password123456".to_string()).unwrap();
        let password2 = Password::new("password123456".to_string()).unwrap();
        let password3 = Password::new("different654321".to_string()).unwrap();
        
        assert_eq!(password1, password2);
        assert_ne!(password1, password3);
    }
}
