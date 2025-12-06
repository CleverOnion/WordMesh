use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_cors::Cors;
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
use controller::assoc::AssocController;
use controller::auth::AuthController;
use controller::sense::SenseController;
use controller::word::WordController;
use middleware::{AuthGuard, RequestId};
use repository::{PgUserRepository, PgWordRepository, Neo4jGraphRepository};
use service::assoc::AssocService;
use service::auth::AuthService;
use service::sense::SenseService;
use service::word::WordService;
use util::{AppError, ResponseBuilder};
use util::token::TokenConfig;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use tokio::time::Duration;

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
        let word_controller = web::Data::new(build_word_controller(shared_settings.clone(), &auth_controller));
        let sense_controller = web::Data::new(build_sense_controller(shared_settings.clone(), &auth_controller));
        let assoc_controller = web::Data::new(build_assoc_controller(shared_settings.clone(), &auth_controller));
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .wrap(RequestId)
            .app_data(web::Data::new(shared_settings.clone()))
            .service(
                web::scope("/api/v1")
                    // Health check endpoint
                    .route("/health", web::get().to(health_check))
                    .configure(|cfg| AuthController::configure(cfg, auth_controller.clone()))
                    .configure(|cfg| WordController::configure(cfg, word_controller.clone()))
                    .configure(|cfg| SenseController::configure(cfg, sense_controller.clone()))
                    .configure(|cfg| AssocController::configure(cfg, assoc_controller.clone())),
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
        .max_connections(db_settings.pool_size)
        .connect_lazy_with(db_settings.connect_options());
    let repository = PgUserRepository::new(pool);
    let auth_settings = &settings.auth;
    let auth_service = AuthService::new(repository, auth_settings, &auth_settings.jwt)
        .expect("failed to initialize auth service");
    AuthController::new(auth_service)
}

fn build_word_controller(
    settings: Arc<Settings>,
    _auth_controller: &AuthController<PgUserRepository>,
) -> WordController<PgWordRepository, Neo4jGraphRepository> {
    // Build token config from settings for AuthGuard
    let jwt_settings = &settings.auth.jwt;
    let token_config = build_token_config_from_settings(jwt_settings)
        .expect("failed to build token config");
    let auth_guard = AuthGuard::new(std::sync::Arc::new(token_config));

    // Initialize PostgreSQL repository
    let db_settings = &settings.database;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_settings.pool_size)
        .connect_lazy_with(db_settings.connect_options());
    let word_repository = PgWordRepository::new(pool);

    // Initialize Neo4j repository
    // Note: Neo4jGraphRepository::from_settings is async, but we're in a sync context
    // We'll need to use a blocking approach or make this async
    // For now, let's create it synchronously using a runtime or make it lazy
    let neo4j_settings = &settings.neo4j;
    let graph = neo4rs::Graph::new(
        &neo4j_settings.uri,
        neo4j_settings.username.clone(),
        neo4j_settings.password.clone(),
    )
    .expect("failed to initialize Neo4j graph");
    let graph_repository = Neo4jGraphRepository::new(
        graph,
        Duration::from_secs(neo4j_settings.query_timeout_seconds),
    );

    // Create WordService
    let word_service = WordService::new(word_repository, graph_repository);

    WordController::new(word_service, auth_guard)
}

fn build_sense_controller(
    settings: Arc<Settings>,
    _auth_controller: &AuthController<PgUserRepository>,
) -> SenseController<PgWordRepository, Neo4jGraphRepository> {
    // Build token config from settings for AuthGuard
    let jwt_settings = &settings.auth.jwt;
    let token_config = build_token_config_from_settings(jwt_settings)
        .expect("failed to build token config");
    let auth_guard = AuthGuard::new(std::sync::Arc::new(token_config));

    // Initialize PostgreSQL repository
    let db_settings = &settings.database;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_settings.pool_size)
        .connect_lazy_with(db_settings.connect_options());
    let word_repository = PgWordRepository::new(pool);

    // Initialize Neo4j repository
    let neo4j_settings = &settings.neo4j;
    let graph = neo4rs::Graph::new(
        &neo4j_settings.uri,
        neo4j_settings.username.clone(),
        neo4j_settings.password.clone(),
    )
    .expect("failed to initialize Neo4j graph");
    let graph_repository = Neo4jGraphRepository::new(
        graph,
        Duration::from_secs(neo4j_settings.query_timeout_seconds),
    );

    // Create SenseService
    let sense_service = SenseService::new(word_repository, graph_repository);

    SenseController::new(sense_service, auth_guard)
}

fn build_assoc_controller(
    settings: Arc<Settings>,
    _auth_controller: &AuthController<PgUserRepository>,
) -> AssocController<PgWordRepository, Neo4jGraphRepository> {
    // Build token config from settings for AuthGuard
    let jwt_settings = &settings.auth.jwt;
    let token_config = build_token_config_from_settings(jwt_settings)
        .expect("failed to build token config");
    let auth_guard = AuthGuard::new(std::sync::Arc::new(token_config));

    // Initialize PostgreSQL repository
    let db_settings = &settings.database;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_settings.pool_size)
        .connect_lazy_with(db_settings.connect_options());
    let word_repository = PgWordRepository::new(pool);

    // Initialize Neo4j repository
    let neo4j_settings = &settings.neo4j;
    let graph = neo4rs::Graph::new(
        &neo4j_settings.uri,
        neo4j_settings.username.clone(),
        neo4j_settings.password.clone(),
    )
    .expect("failed to initialize Neo4j graph");
    let graph_repository = Neo4jGraphRepository::new(
        graph,
        Duration::from_secs(neo4j_settings.query_timeout_seconds),
    );

    // Create AssocService
    let assoc_service = AssocService::new(word_repository, graph_repository);

    AssocController::new(assoc_service, auth_guard)
}

fn build_token_config_from_settings(
    jwt_settings: &config::settings::AuthJwtSettings,
) -> Result<TokenConfig, AppError> {
    let algorithm = match jwt_settings.algorithm.to_uppercase().as_str() {
        "HS256" => Algorithm::HS256,
        "RS256" => Algorithm::RS256,
        _ => {
            return Err(AppError::from(util::error::InternalError::Unknown));
        }
    };

    let (encoding_key, decoding_key) = match algorithm {
        Algorithm::HS256 => {
            let secret = jwt_settings
                .secret
                .clone()
                .ok_or_else(|| AppError::from(util::error::InternalError::Unknown))?;
            (
                EncodingKey::from_secret(secret.as_bytes()),
                DecodingKey::from_secret(secret.as_bytes()),
            )
        }
        Algorithm::RS256 => {
            let private = jwt_settings
                .private_key
                .clone()
                .ok_or_else(|| AppError::from(util::error::InternalError::Unknown))?;
            let public = jwt_settings
                .public_key
                .clone()
                .ok_or_else(|| AppError::from(util::error::InternalError::Unknown))?;
            (
                EncodingKey::from_rsa_pem(private.as_bytes())
                    .map_err(|_| AppError::from(util::error::InternalError::Unknown))?,
                DecodingKey::from_rsa_pem(public.as_bytes())
                    .map_err(|_| AppError::from(util::error::InternalError::Unknown))?,
            )
        }
        _ => unreachable!(),
    };

    Ok(TokenConfig {
        algorithm,
        access_ttl_secs: jwt_settings.access_ttl_secs,
        refresh_ttl_secs: if jwt_settings.refresh_ttl_secs == 0 {
            None
        } else {
            Some(jwt_settings.refresh_ttl_secs)
        },
        encoding_key,
        decoding_key,
        issuer: Some("wordmesh".to_string()),
    })
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
