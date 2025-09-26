use actix_web::{HttpResponse, web};
use std::sync::Arc;

use crate::dto::auth::{LoginRequest, RefreshRequest, RegisterRequest};
use crate::middleware::{AuthGuard, AuthenticatedUser};
use crate::service::auth::AuthService;
use crate::util::{AppError, ResponseBuilder};

#[derive(Clone)]
pub struct AuthController<R>
where
    R: crate::repository::user::UserRepository + Send + Sync + 'static,
{
    service: Arc<AuthService<R>>,
}

impl<R> AuthController<R>
where
    R: crate::repository::user::UserRepository + Send + Sync + 'static,
{
    pub fn new(service: AuthService<R>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    pub fn configure(cfg: &mut web::ServiceConfig, controller: web::Data<AuthController<R>>) {
        let guard = controller.auth_guard();
        cfg.service(
            web::scope("/auth")
                .app_data(controller.clone())
                .route("/register", web::post().to(Self::register))
                .route("/login", web::post().to(Self::login))
                .route("/refresh", web::post().to(Self::refresh))
                .service(
                    web::resource("/profile")
                        .wrap(guard)
                        .route(web::get().to(Self::profile)),
                ),
        );
    }

    async fn register(
        controller: web::Data<AuthController<R>>,
        payload: web::Json<RegisterRequest>,
    ) -> Result<HttpResponse, AppError> {
        let result = controller.service.register(payload.into_inner()).await?;
        ResponseBuilder::ok(result)
    }

    async fn login(
        controller: web::Data<AuthController<R>>,
        payload: web::Json<LoginRequest>,
    ) -> Result<HttpResponse, AppError> {
        let tokens = controller.service.login(payload.into_inner()).await?;
        ResponseBuilder::ok(tokens)
    }

    async fn refresh(
        controller: web::Data<AuthController<R>>,
        payload: web::Json<RefreshRequest>,
    ) -> Result<HttpResponse, AppError> {
        let tokens = controller.service.refresh(payload.into_inner()).await?;
        ResponseBuilder::ok(tokens)
    }

    async fn profile(
        controller: web::Data<AuthController<R>>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let profile = controller.service.profile(identity.user_id).await?;
        ResponseBuilder::ok(profile)
    }

    fn auth_guard(&self) -> AuthGuard {
        AuthGuard::new(self.service.token_config())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use async_trait::async_trait;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    use crate::config::settings::{AuthJwtSettings, AuthPasswordSettings, AuthSettings};
    use crate::domain::User;
    use crate::repository::user::{NewUser, RepositoryError, UserRepository};
    use crate::service::auth::AuthService;

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
                    crate::domain::UserDomainError::InvalidUsername(
                        crate::domain::UsernameValidationError::InvalidFormat,
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
                access_ttl_secs: 60,
                refresh_ttl_secs: 120,
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

    fn service() -> AuthService<InMemoryUserRepository> {
        let settings = default_settings();
        AuthService::new(InMemoryUserRepository::default(), &settings, &settings.jwt).unwrap()
    }

    #[actix_rt::test]
    async fn register_endpoint_returns_profile() {
        let controller = web::Data::new(AuthController::new(service()));
        let app = test::init_service(
            App::new().configure(|cfg| AuthController::configure(cfg, controller.clone())),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "user_register", "password": "password123" }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["data"]["username"], "user_register");
    }

    #[actix_rt::test]
    async fn login_endpoint_returns_tokens() {
        let controller = web::Data::new(AuthController::new(service()));
        let app = test::init_service(
            App::new().configure(|cfg| AuthController::configure(cfg, controller.clone())),
        )
        .await;

        // register first
        let register = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "user_login", "password": "password123" }))
            .to_request();
        let _ = test::call_service(&app, register).await;

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "username": "user_login", "password": "password123" }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["data"]["access_token"].as_str().unwrap().len() > 10);
    }

    #[actix_rt::test]
    async fn profile_requires_identity() {
        let controller = web::Data::new(AuthController::new(service()));
        let app = test::init_service(
            App::new().configure(|cfg| AuthController::configure(cfg, controller.clone())),
        )
        .await;

        let register = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "user_profile", "password": "password123" }))
            .to_request();
        let resp = test::call_service(&app, register).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        let user_id = body["data"]["id"].as_i64().unwrap();

        let login = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "username": "user_profile", "password": "password123" }))
            .to_request();
        let login_resp = test::call_service(&app, login).await;
        assert!(login_resp.status().is_success());
        let login_body: serde_json::Value = test::read_body_json(login_resp).await;
        let token = login_body["data"]["access_token"].as_str().unwrap();

        let req = test::TestRequest::get()
            .uri("/auth/profile")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["data"]["id"].as_i64().unwrap(), user_id);
    }
}
