use actix_web::{App, test, web};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use wordmesh_backend::config::settings::{AuthJwtSettings, AuthPasswordSettings, AuthSettings};
use wordmesh_backend::controller::auth::AuthController;
use wordmesh_backend::domain::User;
use wordmesh_backend::repository::user::{NewUser, RepositoryError, UserRepository};
use wordmesh_backend::service::auth::AuthService;

#[derive(Default, Clone)]
struct InMemoryUserRepository {
    users: Arc<RwLock<HashMap<i64, User>>>,
    username_index: Arc<RwLock<HashMap<String, i64>>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, RepositoryError> {
        let mut users = self.users.write().await;
        let mut username_idx = self.username_index.write().await;

        if username_idx.contains_key(&new_user.username) {
            return Err(RepositoryError::Domain(
                wordmesh_backend::domain::UserDomainError::InvalidUsername(
                    wordmesh_backend::domain::UsernameValidationError::InvalidFormat,
                ),
            ));
        }

        let id = (users.len() + 1) as i64;
        let user = User::new(
            id,
            new_user.username.clone(),
            new_user.password_hash,
            Utc::now(),
        )
        .unwrap();
        username_idx.insert(user.username.clone(), user.id);
        users.insert(id, user.clone());
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let username_idx = self.username_index.read().await;
        let users = self.users.read().await;
        Ok(username_idx
            .get(username)
            .and_then(|id| users.get(id))
            .cloned())
    }

    async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, RepositoryError> {
        let users = self.users.read().await;
        Ok(users.get(&user_id).cloned())
    }
}

fn default_settings() -> AuthSettings {
    AuthSettings {
        enabled: true,
        jwt: AuthJwtSettings {
            algorithm: "HS256".into(),
            access_ttl_secs: 3600,
            refresh_ttl_secs: 604800,
            secret: Some("secretsecretsecretsecret".into()),
            private_key: None,
            public_key: None,
        },
        password: AuthPasswordSettings {
            min_length: 8,
            require_complexity: false,
        },
    }
}

#[actix_rt::test]
async fn register_login_profile_flow() {
    let settings = default_settings();
    let service =
        AuthService::new(InMemoryUserRepository::default(), &settings, &settings.jwt).unwrap();
    let controller = web::Data::new(AuthController::new(service));
    let controller_cfg = controller.clone();

    let app = test::init_service(
        App::new().configure(move |cfg| AuthController::configure(cfg, controller_cfg.clone())),
    )
    .await;

    // Register
    let register_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "integration_user", "password": "password123" }))
            .to_request(),
    )
    .await;
    assert!(register_resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(register_resp).await;
    let user_id = body["data"]["id"].as_i64().unwrap();

    // Login
    let login_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "username": "integration_user", "password": "password123" }))
            .to_request(),
    )
    .await;
    assert!(login_resp.status().is_success());
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let access_token = login_body["data"]["access_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Profile with bearer token
    let profile_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/auth/profile")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .to_request(),
    )
    .await;
    assert!(profile_resp.status().is_success());
    let profile_body: serde_json::Value = test::read_body_json(profile_resp).await;
    assert_eq!(profile_body["data"]["id"].as_i64().unwrap(), user_id);
}

#[actix_rt::test]
async fn refresh_and_unauthorized_flow() {
    let settings = default_settings();
    let service =
        AuthService::new(InMemoryUserRepository::default(), &settings, &settings.jwt).unwrap();
    let controller = web::Data::new(AuthController::new(service));
    let controller_cfg = controller.clone();

    let app = test::init_service(
        App::new().configure(move |cfg| AuthController::configure(cfg, controller_cfg.clone())),
    )
    .await;

    // Register and login
    let _ = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "refresh_user", "password": "password123" }))
            .to_request(),
    )
    .await;

    let login_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "username": "refresh_user", "password": "password123" }))
            .to_request(),
    )
    .await;
    assert!(login_resp.status().is_success());
    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let refresh_token = login_body["data"]["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();

    // Refresh token
    let refresh_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/auth/refresh")
            .set_json(&json!({ "refresh_token": refresh_token }))
            .to_request(),
    )
    .await;
    assert!(refresh_resp.status().is_success());
    let refresh_body: serde_json::Value = test::read_body_json(refresh_resp).await;
    assert!(refresh_body["data"]["access_token"].as_str().unwrap().len() > 10);

    // Unauthorized profile should return auth error payload
    let unauth_resp = test::call_service(
        &app,
        test::TestRequest::get().uri("/auth/profile").to_request(),
    )
    .await;
    assert!(unauth_resp.status().is_success());
    let unauth_body: serde_json::Value = test::read_body_json(unauth_resp).await;
    assert_eq!(unauth_body["code"], 4011);
}
