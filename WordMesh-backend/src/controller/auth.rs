use actix_web::{web, HttpRequest, HttpResponse, FromRequest};
use actix_web::dev::Payload;
use serde::Deserialize;
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::dto::auth::{LoginRequest, RefreshRequest, RegisterRequest};
use crate::service::auth::AuthService;
use crate::util::error::{AuthFlowError, BusinessError};
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

    pub fn configure(cfg: &mut web::ServiceConfig, controller: AuthController<R>) {
        let controller = web::Data::new(controller);
        cfg.app_data(controller.clone())
            .route("/auth/register", web::post().to(Self::register))
            .route("/auth/login", web::post().to(Self::login))
            .route("/auth/refresh", web::post().to(Self::refresh))
            .route("/auth/profile", web::get().to(Self::profile));
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
        identity: Identity,
    ) -> Result<HttpResponse, AppError> {
        let user_id = identity
            .user_id
            .ok_or_else(|| AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)))?;
        let profile = controller.service.profile(user_id).await?;
        ResponseBuilder::ok(profile)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Identity {
    pub user_id: Option<i64>,
}

impl FromRequest for Identity {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(header) = req.headers().get("X-User-Id") {
            if let Ok(value) = header.to_str() {
                if let Ok(id) = value.parse::<i64>() {
                    return ready(Ok(Identity { user_id: Some(id) }));
                }
            }
        }
        ready(Ok(Identity { user_id: None }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
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
                return Err(RepositoryError::Domain(crate::domain::UserDomainError::InvalidUsername(
                    crate::domain::UsernameValidationError::InvalidFormat,
                )));
            }
            let id = (users.len() + 1) as i64;
            let user = User::new(id, new_user.username.clone(), new_user.password_hash, Utc::now()).unwrap();
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
        let controller = AuthController::new(service());
        let app = test::init_service(App::new().configure(|cfg| AuthController::configure(cfg, controller.clone()))).await;

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
        let controller = AuthController::new(service());
        let app = test::init_service(App::new().configure(|cfg| AuthController::configure(cfg, controller.clone()))).await;

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
        let controller = AuthController::new(service());
        let app = test::init_service(App::new().configure(|cfg| AuthController::configure(cfg, controller.clone()))).await;

        let register = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({ "username": "user_profile", "password": "password123" }))
            .to_request();
        let resp = test::call_service(&app, register).await;
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        let user_id = body["data"]["id"].as_i64().unwrap();

        let req = test::TestRequest::get()
            .uri("/auth/profile")
            .insert_header(("X-User-Id", user_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
