use crate::infrastructure::http::controllers::{auth_handler, health_handler};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/auth", web::post().to(auth_handler))
            .route("/health", web::get().to(health_handler))
    );
}
