mod domain;
mod infrastructure;

use domain::auth_service::AuthService;
use domain::config::Settings;
use infrastructure::http::configure_routes;
use infrastructure::AutheliaHttpClient;

use actix_web::{middleware::Logger, web, App, HttpServer};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::from_env().expect("Failed to load configuration");

    settings.validate().expect("Invalid configuration");

    init_tracing(&settings);

    tracing::info!("Starting sftpgo-authelia-totp-hook");
    tracing::info!(
        authelia_base_url = %settings.authelia_base_url,
        http_bind = %settings.http_bind,
        http_client_timeout_ms = %settings.http_client_timeout.as_millis(),
        log_level = %settings.log_level,
        tls_insecure = %settings.tls_insecure,
        "Configuration loaded"
    );

    let authelia_client = Arc::new(
        AutheliaHttpClient::new(&settings).expect("Failed to create Authelia HTTP client"),
    );

    let auth_service = Arc::new(AuthService::new(authelia_client));

    tracing::info!("Starting HTTP server on {}", settings.http_bind);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(auth_service.clone()))
            .wrap(Logger::default())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(configure_routes)
    })
    .bind(&settings.http_bind)?
    .run()
    .await
}

fn init_tracing(settings: &Settings) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("sftpgo_authelia_totp_hook={}", settings.log_level).into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
