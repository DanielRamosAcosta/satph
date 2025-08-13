use crate::domain::AuthError;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username {
    value: String,
}

impl Username {
    pub fn new(value: String) -> Result<Self, AuthError> {
        Self::validate(&value)?;
        Ok(Self { value })
    }

    fn validate(value: &str) -> Result<(), AuthError> {
        if value.is_empty() {
            return Err(AuthError::input_invalid("Username cannot be empty"));
        }

        if value.len() > 128 {
            return Err(AuthError::input_invalid(
                "Username too long (max 128 characters)",
            ));
        }

        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn into_string(self) -> String {
        self.value
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl From<Username> for String {
    fn from(username: Username) -> Self {
        username.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_valid() {
        let username = Username::new("alice".to_string()).unwrap();
        assert_eq!(username.as_str(), "alice");
        assert_eq!(username.to_string(), "alice");
    }

    #[test]
    fn test_username_valid_max_length() {
        let long_username = "a".repeat(128);
        let username = Username::new(long_username.clone()).unwrap();
        assert_eq!(username.as_str(), &long_username);
    }

    #[test]
    fn test_username_empty() {
        let result = Username::new("".to_string());
        assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
        if let Err(AuthError::InputInvalid { message }) = result {
            assert_eq!(message, "Username cannot be empty");
        }
    }

    #[test]
    fn test_username_too_long() {
        let long_username = "a".repeat(129);
        let result = Username::new(long_username);
        assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
        if let Err(AuthError::InputInvalid { message }) = result {
            assert_eq!(message, "Username too long (max 128 characters)");
        }
    }

    #[test]
    fn test_username_display() {
        let username = Username::new("testuser".to_string()).unwrap();
        assert_eq!(format!("{}", username), "testuser");
    }

    #[test]
    fn test_username_as_ref() {
        let username = Username::new("testuser".to_string()).unwrap();
        let as_ref: &str = username.as_ref();
        assert_eq!(as_ref, "testuser");
    }

    #[test]
    fn test_username_into_string() {
        let username = Username::new("testuser".to_string()).unwrap();
        let string_value: String = username.into();
        assert_eq!(string_value, "testuser");
    }

    #[test]
    fn test_username_equality() {
        let username1 = Username::new("alice".to_string()).unwrap();
        let username2 = Username::new("alice".to_string()).unwrap();
        let username3 = Username::new("bob".to_string()).unwrap();
        
        assert_eq!(username1, username2);
        assert_ne!(username1, username3);
    }
}
