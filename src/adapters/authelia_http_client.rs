use crate::config::Settings;
use crate::domain::auth_service::Authelia;
use crate::domain::{AuthError, SessionCookie};
use async_trait::async_trait;
use reqwest::{Client, Response, StatusCode};
use serde_json::json;

pub struct AutheliaHttpClient {
    client: Client,
    base_url: String,
}

impl AutheliaHttpClient {
    pub fn new(settings: &Settings) -> Result<Self, AuthError> {
        let mut client_builder = Client::builder()
            .timeout(settings.http_client_timeout)
            .cookie_store(true); // Enable cookie jar

        if settings.tls_insecure {
            tracing::warn!("TLS verification disabled - not for production use");
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        let client = client_builder
            .build()
            .map_err(|e| AuthError::unexpected(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: settings.authelia_base_url.clone(),
        })
    }

    fn extract_session_cookie(&self, response: &Response) -> Option<String> {
        for header_value in response.headers().get_all("set-cookie") {
            if let Ok(cookie_str) = header_value.to_str() {
                if cookie_str.starts_with("authelia_session=") {
                    // Extract the cookie value before the first semicolon
                    let cookie_value = cookie_str.split(';').next()?.split('=').nth(1)?;
                    return Some(cookie_value.to_string());
                }
            }
        }
        None
    }

    async fn handle_response_error(response: Response) -> AuthError {
        let status = response.status();

        match status {
            StatusCode::UNAUTHORIZED => {
                if response.url().path().contains("firstfactor") {
                    AuthError::FirstFactorUnauthorized
                } else {
                    AuthError::SecondFactorUnauthorized
                }
            }
            StatusCode::BAD_REQUEST => {
                let error_text = response.text().await.unwrap_or_default();
                AuthError::input_invalid(format!("Bad request: {}", error_text))
            }
            StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => AuthError::Timeout,
            s if s.is_server_error() => AuthError::UpstreamUnavailable,
            _ => {
                let error_text = response.text().await.unwrap_or_default();
                AuthError::unexpected(format!("Unexpected status {}: {}", status, error_text))
            }
        }
    }
}

#[async_trait]
impl Authelia for AutheliaHttpClient {
    async fn first_factor(
        &self,
        username: &str,
        password: &str,
    ) -> Result<SessionCookie, AuthError> {
        let url = format!("{}/api/firstfactor", self.base_url);

        let body = json!({
            "username": username,
            "password": password
        });

        tracing::debug!(
            username = %username,
            url = %url,
            "Sending first factor request"
        );

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    tracing::warn!("First factor request timeout");
                    AuthError::Timeout
                } else if e.is_connect() {
                    tracing::error!("Failed to connect to Authelia: {}", e);
                    AuthError::UpstreamUnavailable
                } else {
                    tracing::error!("First factor request failed: {}", e);
                    AuthError::unexpected(format!("Request failed: {}", e))
                }
            })?;

        if response.status().is_success() {
            if let Some(session_value) = self.extract_session_cookie(&response) {
                tracing::debug!("First factor authentication successful");
                Ok(SessionCookie {
                    value: session_value,
                })
            } else {
                tracing::warn!("First factor successful but no session cookie found");
                Err(AuthError::unexpected(
                    "No session cookie in successful first factor response".to_string(),
                ))
            }
        } else {
            tracing::warn!(
                status = %response.status(),
                "First factor authentication failed"
            );
            Err(Self::handle_response_error(response).await)
        }
    }

    async fn second_factor_totp(
        &self,
        session: &SessionCookie,
        token: &str,
    ) -> Result<(), AuthError> {
        let url = format!("{}/api/secondfactor/totp", self.base_url);

        let body = json!({
            "token": token
        });

        tracing::debug!(
            url = %url,
            "Sending second factor TOTP request"
        );

        let response = self
            .client
            .post(&url)
            .header("Cookie", format!("authelia_session={}", session.value))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    tracing::warn!("Second factor request timeout");
                    AuthError::Timeout
                } else if e.is_connect() {
                    tracing::error!("Failed to connect to Authelia: {}", e);
                    AuthError::UpstreamUnavailable
                } else {
                    tracing::error!("Second factor request failed: {}", e);
                    AuthError::unexpected(format!("Request failed: {}", e))
                }
            })?;

        if response.status().is_success() {
            tracing::debug!("Second factor TOTP authentication successful");
            Ok(())
        } else {
            tracing::warn!(
                status = %response.status(),
                "Second factor TOTP authentication failed"
            );
            Err(Self::handle_response_error(response).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use std::time::Duration;

    #[test]
    fn test_extract_session_cookie() {
        let settings = Settings {
            authelia_base_url: "http://localhost".to_string(),
            http_bind: "0.0.0.0:8080".to_string(),
            http_client_timeout: Duration::from_millis(5000),
            log_level: "info".to_string(),
            tls_insecure: false,
        };

        let client = AutheliaHttpClient::new(&settings).unwrap();

        // Mock response would need a real HTTP response, so this is a basic structure test
        assert_eq!(client.base_url, "http://localhost");
    }
}
