use actix_web::{App, HttpServer, middleware::Logger, web};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod application;
mod config;
mod controller;
mod domain;
mod dto;
mod event;
mod middleware;
mod repository;
mod service;
mod util;

use config::Settings;
use middleware::RequestId;
use util::{AppError, ResponseBuilder};

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wordmesh=debug,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Arc::new(Settings::load().unwrap_or_else(|_| Settings::default()));

    tracing::info!(
        "Starting WordMesh backend server on {}:{}",
        settings.application.host,
        settings.application.port
    );

    // Start HTTP server
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let shared_settings = settings.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(RequestId)
            .app_data(web::Data::new(shared_settings.clone()))
            .service(
                web::scope("/api/v1")
                    // Health check endpoint
                    .route("/health", web::get().to(health_check)),
            )
    })
    .bind(address)
    .map_err(AppError::from)?
    .run()
    .await
    .map_err(AppError::from)
}

async fn health_check() -> Result<actix_web::HttpResponse, AppError> {
    #[derive(serde::Serialize)]
    struct HealthStatus {
        status: String,
        service: String,
        version: String,
    }

    let health_data = HealthStatus {
        status: "healthy".to_string(),
        service: "WordMesh Backend".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(ResponseBuilder::ok(health_data)?)
}
