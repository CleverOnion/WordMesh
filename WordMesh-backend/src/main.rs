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
use controller::auth::AuthController;
use middleware::RequestId;
use repository::PgUserRepository;
use service::auth::AuthService;
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
        let auth_controller = web::Data::new(build_auth_controller(shared_settings.clone()));
        App::new()
            .wrap(Logger::default())
            .wrap(RequestId)
            .app_data(web::Data::new(shared_settings.clone()))
            .service(
                web::scope("/api/v1")
                    // Health check endpoint
                    .route("/health", web::get().to(health_check))
                    .configure(|cfg| AuthController::configure(cfg, auth_controller.clone())),
            )
    })
    .bind(address)
    .map_err(AppError::from)?
    .run()
    .await
    .map_err(AppError::from)
}

fn build_auth_controller(settings: Arc<Settings>) -> AuthController<PgUserRepository> {
    let db_settings = &settings.database;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_settings.max_connections)
        .connect_lazy_with(db_settings.connect_options());
    let repository = PgUserRepository::new(pool);
    let auth_settings = &settings.auth;
    let auth_service = AuthService::new(repository, auth_settings, &auth_settings.jwt)
        .expect("failed to initialize auth service");
    AuthController::new(auth_service)
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
