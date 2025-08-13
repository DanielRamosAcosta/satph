use crate::domain::auth_service::AuthService;
use crate::domain::{AuthError, AuthRequest, AuthResult, Password, Protocol, Username};
use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthRequestDto {
    pub username: String,
    pub password: String,
    pub ip: String,
    pub protocol: Protocol,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponseDto {
    pub status: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponseDto {
    pub status: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponseDto {
    pub status: String,
    pub service: String,
    pub version: String,
}

impl AuthResponseDto {
    pub fn success() -> Self {
        Self { status: 1 }
    }

    pub fn failure() -> Self {
        Self { status: 0 }
    }
}

impl ErrorResponseDto {
    pub fn new(message: Option<String>) -> Self {
        Self { status: 0, message }
    }
}

pub async fn health_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponseDto {
        status: "healthy".to_string(),
        service: "sftpgo-authelia-totp-hook".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

pub async fn auth_handler(
    auth_request: web::Json<AuthRequestDto>,
    auth_service: web::Data<Arc<AuthService>>,
) -> ActixResult<HttpResponse> {
    let trace_id = Uuid::new_v4();
    let span = tracing::info_span!("auth_request", trace_id = %trace_id);

    async move {
        tracing::info!("Authentication request received");

        let domain_request = match validate_and_convert_request(auth_request.into_inner()) {
            Ok(req) => req,
            Err(error) => {
                tracing::warn!(error = %error, "Invalid request");
                return Ok(
                    HttpResponse::BadRequest().json(ErrorResponseDto::new(Some(error.to_string())))
                );
            }
        };
        match auth_service.authenticate(domain_request).await {
            Ok(AuthResult { success: true }) => {
                tracing::info!("Authentication successful");
                Ok(HttpResponse::Ok().json(AuthResponseDto::success()))
            }
            Ok(AuthResult { success: false }) => {
                tracing::info!("Authentication failed");
                Ok(HttpResponse::Unauthorized().json(AuthResponseDto::failure()))
            }
            Err(error) => handle_auth_error(error),
        }
    }
    .instrument(span)
    .await
}

fn validate_and_convert_request(dto: AuthRequestDto) -> Result<AuthRequest, AuthError> {
    let username = Username::new(dto.username)?;
    let password = Password::new(dto.password)?;

    let ip_addr: IpAddr = dto
        .ip
        .parse()
        .map_err(|_| AuthError::input_invalid("Invalid IP address format"))?;

    Ok(AuthRequest {
        username,
        password,
        ip: ip_addr,
        protocol: dto.protocol,
    })
}

fn handle_auth_error(error: AuthError) -> ActixResult<HttpResponse> {
    match error {
        AuthError::InputInvalid { message } => {
            tracing::warn!(error = %message, "Input validation failed");
            Ok(HttpResponse::BadRequest().json(ErrorResponseDto::new(Some(message))))
        }
        AuthError::FirstFactorUnauthorized => {
            tracing::info!("First factor authentication failed");
            Ok(HttpResponse::Unauthorized().json(AuthResponseDto::failure()))
        }
        AuthError::SecondFactorUnauthorized => {
            tracing::info!("Second factor authentication failed");
            Ok(HttpResponse::Unauthorized().json(AuthResponseDto::failure()))
        }
        AuthError::UpstreamUnavailable | AuthError::Timeout => {
            tracing::error!(error = %error, "Upstream service error");
            Ok(HttpResponse::InternalServerError().json(AuthResponseDto::failure()))
        }
        AuthError::Unexpected { message } => {
            tracing::error!(error = %message, "Unexpected error");
            Ok(HttpResponse::InternalServerError().json(AuthResponseDto::failure()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth_service::AuthService;
    use crate::domain::{AuthError, Authelia, SessionCookie};
    use actix_web::{test, web, App};
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockAutheliaPort {
        should_succeed: bool,
    }

    #[async_trait]
    impl Authelia for MockAutheliaPort {
        async fn first_factor(
            &self,
            _username: &Username,
            _password: &str,
        ) -> Result<SessionCookie, AuthError> {
            if self.should_succeed {
                Ok(SessionCookie {
                    value: "mock_session".to_string(),
                })
            } else {
                Err(AuthError::FirstFactorUnauthorized)
            }
        }

        async fn second_factor_totp(
            &self,
            _session: &SessionCookie,
            _token: &str,
        ) -> Result<(), AuthError> {
            if self.should_succeed {
                Ok(())
            } else {
                Err(AuthError::SecondFactorUnauthorized)
            }
        }
    }

    #[actix_web::test]
    async fn test_auth_handler_success() {
        let mock_port = Arc::new(MockAutheliaPort {
            should_succeed: true,
        });
        let auth_service = Arc::new(AuthService::new(mock_port));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(auth_service))
                .route("/auth", web::post().to(auth_handler)),
        )
        .await;

        let req_body = AuthRequestDto {
            username: "testuser".to_string(),
            password: "password123456".to_string(),
            ip: "192.168.1.1".to_string(),
            protocol: Protocol::SSH,
        };

        let req = test::TestRequest::post()
            .uri("/auth")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: AuthResponseDto = test::read_body_json(resp).await;
        assert_eq!(body.status, 1);
    }

    #[actix_web::test]
    async fn test_auth_handler_failure() {
        let mock_port = Arc::new(MockAutheliaPort {
            should_succeed: false,
        });
        let auth_service = Arc::new(AuthService::new(mock_port));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(auth_service))
                .route("/auth", web::post().to(auth_handler)),
        )
        .await;

        let req_body = AuthRequestDto {
            username: "testuser".to_string(),
            password: "password123456".to_string(),
            ip: "192.168.1.1".to_string(),
            protocol: Protocol::SSH,
        };

        let req = test::TestRequest::post()
            .uri("/auth")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        let body: AuthResponseDto = test::read_body_json(resp).await;
        assert_eq!(body.status, 0);
    }

    #[actix_web::test]
    async fn test_auth_handler_invalid_input() {
        let mock_port = Arc::new(MockAutheliaPort {
            should_succeed: true,
        });
        let auth_service = Arc::new(AuthService::new(mock_port));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(auth_service))
                .route("/auth", web::post().to(auth_handler)),
        )
        .await;

        let req_body = AuthRequestDto {
            username: "".to_string(),
            password: "password123456".to_string(),
            ip: "192.168.1.1".to_string(),
            protocol: Protocol::SSH,
        };

        let req = test::TestRequest::post()
            .uri("/auth")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: ErrorResponseDto = test::read_body_json(resp).await;
        assert_eq!(body.status, 0);
        assert!(body.message.is_some());
    }

    /*
        #[test]
        fn test_validate_and_convert_request_success() {
            let dto = AuthRequestDto {
                username: "testuser".to_string(),
                password: "password123456".to_string(),
                ip: "192.168.1.1".to_string(),
                protocol: Protocol::SSH,
            };

            let result = validate_and_convert_request(dto).unwrap();
            assert_eq!(result.username.as_str(), "testuser");
            assert_eq!(result.password, "password123456");
            assert_eq!(result.protocol, Protocol::SSH);
        }

        #[test]
        fn test_validate_and_convert_request_invalid_ip() {
            let dto = AuthRequestDto {
                username: "testuser".to_string(),
                password: "password123456".to_string(),
                ip: "invalid-ip".to_string(),
                protocol: Protocol::SSH,
            };

            let result = validate_and_convert_request(dto);
            assert!(matches!(result, Err(AuthError::InputInvalid { .. })));
        }
    }
        }
        */
}
