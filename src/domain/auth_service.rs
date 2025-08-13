use crate::domain::{AuthError, AuthRequest, AuthResult, Authelia};

#[cfg(test)]
use crate::domain::{SessionCookie, Username};
#[cfg(test)]
use async_trait::async_trait;
use std::sync::Arc;

pub struct AuthService {
    authelia: Arc<dyn Authelia>,
}

impl AuthService {
    pub fn new(authelia: Arc<dyn Authelia>) -> Self {
        Self { authelia }
    }

    pub async fn authenticate(&self, request: AuthRequest) -> Result<AuthResult, AuthError> {
        tracing::info!(
            username = %request.username,
            ip = %request.ip,
            protocol = %request.protocol,
            "Starting authentication"
        );

        let session_cookie = self
            .authelia
            .first_factor(&request.username, &request.password.first_factor_password())
            .await?;

        tracing::debug!("First factor authentication successful");

        self.authelia
            .second_factor_totp(&session_cookie, &request.password.totp_token())
            .await?;

        tracing::info!(
            username = %request.username,
            ip = %request.ip,
            protocol = %request.protocol,
            "Authentication successful"
        );

        Ok(AuthResult::success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Password, Protocol};
    use std::net::IpAddr;
    use std::str::FromStr;

    struct MockAutheliaPort {
        first_factor_result: Result<SessionCookie, AuthError>,
        second_factor_result: Result<(), AuthError>,
    }

    impl MockAutheliaPort {
        fn success() -> Self {
            Self {
                first_factor_result: Ok(SessionCookie {
                    value: "mock_session".to_string(),
                }),
                second_factor_result: Ok(()),
            }
        }

        fn first_factor_fail() -> Self {
            Self {
                first_factor_result: Err(AuthError::FirstFactorUnauthorized),
                second_factor_result: Ok(()),
            }
        }

        fn second_factor_fail() -> Self {
            Self {
                first_factor_result: Ok(SessionCookie {
                    value: "mock_session".to_string(),
                }),
                second_factor_result: Err(AuthError::SecondFactorUnauthorized),
            }
        }
    }

    #[async_trait]
    impl Authelia for MockAutheliaPort {
        async fn first_factor(
            &self,
            _username: &Username,
            _password: &str,
        ) -> Result<SessionCookie, AuthError> {
            self.first_factor_result.clone()
        }

        async fn second_factor_totp(
            &self,
            _session: &SessionCookie,
            _token: &str,
        ) -> Result<(), AuthError> {
            self.second_factor_result.clone()
        }
    }

    fn create_test_request() -> AuthRequest {
        AuthRequest {
            username: Username::new("testuser".to_string()).unwrap(),
            password: Password::new("myPassword123456".to_string()).unwrap(),
            ip: IpAddr::from_str("192.168.1.1").unwrap(),
            protocol: Protocol::SSH,
        }
    }

    #[tokio::test]
    async fn test_authenticate_success() {
        let service = AuthService::new(Arc::new(MockAutheliaPort::success()));
        let request = create_test_request();

        let result = service.authenticate(request).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_authenticate_first_factor_fail() {
        let service = AuthService::new(Arc::new(MockAutheliaPort::first_factor_fail()));
        let request = create_test_request();

        let result = service.authenticate(request).await;
        assert!(matches!(result, Err(AuthError::FirstFactorUnauthorized)));
    }

    #[tokio::test]
    async fn test_authenticate_second_factor_fail() {
        let service = AuthService::new(Arc::new(MockAutheliaPort::second_factor_fail()));
        let request = create_test_request();

        let result = service.authenticate(request).await;
        assert!(matches!(result, Err(AuthError::SecondFactorUnauthorized)));
    }
}
