use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::config::settings::{AuthJwtSettings, AuthPasswordSettings, AuthSettings};
use crate::domain::user::USERNAME_REGEX;
use crate::domain::{HashedPassword, User};
use crate::dto::auth::{AuthTokens, LoginRequest, ProfileResponse, RefreshRequest, RegisterRequest};
use crate::repository::user::{NewUser, RepositoryError, UserRepository};
use crate::util::error::{AuthFlowError, BusinessError, InternalError, ValidationField};
use crate::util::password::{hash_password, verify_password, PasswordError};
use crate::util::token::{generate_access_token, generate_refresh_token, validate_token, TokenConfig, TokenError};
use crate::util::AppError;

#[derive(Clone)]
pub struct AuthService<R: UserRepository + Send + Sync + 'static> {
    repository: Arc<R>,
    token_config: Arc<TokenConfig>,
    password_cost: u32,
    pub auth_enabled: bool,
}

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("validation failed")]
    Validation(Vec<ValidationField>),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("token error: {0}")]
    Token(#[from] TokenError),
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

impl From<AuthServiceError> for AppError {
    fn from(err: AuthServiceError) -> Self {
        match err {
            AuthServiceError::Validation(fields) => AppError::from(BusinessError::Validation(fields)),
            AuthServiceError::InvalidCredentials => AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)),
            AuthServiceError::Token(TokenError::RefreshDisabled) => AppError::from(BusinessError::Auth(AuthFlowError::RefreshDisabled)),
            AuthServiceError::Token(TokenError::Decode(_)) => AppError::from(BusinessError::Auth(AuthFlowError::TokenInvalid)),
            AuthServiceError::Token(TokenError::Encode(_)) => AppError::from(BusinessError::Auth(AuthFlowError::TokenInvalid)),
            AuthServiceError::Repository(err) => match err {
                RepositoryError::Database(_) => AppError::from(InternalError::Unknown),
                RepositoryError::Domain(_) => AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)),
            },
        }
    }
}

impl<R> AuthService<R>
where
    R: UserRepository + Send + Sync + 'static,
{
    pub fn new(repository: R, auth_settings: &AuthSettings, jwt_settings: &AuthJwtSettings) -> Result<Self, AppError> {
        let token_config = build_token_config(jwt_settings)?;
        Ok(Self {
            repository: Arc::new(repository),
            token_config: Arc::new(token_config),
            password_cost: auth_settings.password.min_length.max(8) as u32,
            auth_enabled: auth_settings.enabled,
        })
    }

    pub async fn register(&self, payload: RegisterRequest) -> Result<ProfileResponse, AppError> {
        self.ensure_enabled()?;
        payload
            .validate()
            .map_err(|err| AppError::from(BusinessError::Validation(validation_errors(err))))?;

        let hashed = hash_password(&payload.password, self.password_cost).map_err(map_password_error)?;
        let password_hash = HashedPassword::new(hashed).map_err(|_| AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)))?;

        let new_user = NewUser {
            username: payload.username.clone(),
            password_hash,
        };

        let user = self
            .repository
            .create_user(new_user)
            .await
            .map_err(map_repository_error)?;

        Ok(ProfileResponse {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
        })
    }

    pub async fn login(&self, payload: LoginRequest) -> Result<AuthTokens, AppError> {
        self.ensure_enabled()?;
        payload
            .validate()
            .map_err(|err| AppError::from(BusinessError::Validation(validation_errors(err))))?;

        let user = self
            .repository
            .find_by_username(&payload.username)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)))?;

        let password_ok = verify_password(&payload.password, user.password_hash.as_str()).map_err(map_password_error)?;
        if !password_ok {
            return Err(AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)));
        }

        let access_token = generate_access_token(&self.token_config, &user.id.to_string(), None, None)
            .map_err(map_token_error)?;
        let refresh_token = self
            .token_config
            .refresh_ttl_secs
            .map(|_| generate_refresh_token(&self.token_config, &user.id.to_string(), None).map_err(map_token_error))
            .transpose()?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh(&self, payload: RefreshRequest) -> Result<AuthTokens, AppError> {
        self.ensure_enabled()?;
        payload
            .validate()
            .map_err(|err| AppError::from(BusinessError::Validation(validation_errors(err))))?;

        let claims = validate_token(&self.token_config, &payload.refresh_token)
            .map_err(map_token_error)?;

        let user_id = claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::from(BusinessError::Auth(AuthFlowError::TokenInvalid)))?;

        let user = self
            .repository
            .find_by_id(user_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)))?;

        let access_token = generate_access_token(&self.token_config, &user.id.to_string(), claims.scope.clone(), claims.request_id.clone())
            .map_err(map_token_error)?;
        let refresh_token = self
            .token_config
            .refresh_ttl_secs
            .map(|_| generate_refresh_token(&self.token_config, &user.id.to_string(), claims.request_id.clone()).map_err(map_token_error))
            .transpose()?;

        Ok(AuthTokens {
            access_token,
            refresh_token,
        })
    }

    pub async fn profile(&self, user_id: i64) -> Result<ProfileResponse, AppError> {
        self.ensure_enabled()?;
        let user = self
            .repository
            .find_by_id(user_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)))?;

        Ok(ProfileResponse {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
        })
    }

    fn ensure_enabled(&self) -> Result<(), AppError> {
        if !self.auth_enabled {
            Err(AppError::from(BusinessError::Auth(AuthFlowError::RefreshDisabled)))
        } else {
            Ok(())
        }
    }
}

fn map_repository_error(err: RepositoryError) -> AppError {
    match err {
        RepositoryError::Database(_) => AppError::from(InternalError::Unknown),
        RepositoryError::Domain(_) => AppError::from(BusinessError::Auth(AuthFlowError::InvalidCredentials)),
    }
}

fn map_password_error(err: PasswordError) -> AppError {
    match err {
        PasswordError::Empty => AppError::from(BusinessError::Validation(vec![ValidationField {
            field: "password".into(),
            message: "密码不能为空".into(),
        }])),
        PasswordError::Hash(_) | PasswordError::Verify => AppError::from(InternalError::Unknown),
    }
}

fn validation_errors(err: ValidationErrors) -> Vec<ValidationField> {
    let mut fields = Vec::new();
    for (field, errors) in err.field_errors() {
        for error in errors {
            let message = error.message.clone().unwrap_or_else(|| "参数错误".into());
            fields.push(ValidationField {
                field: field.to_string(),
                message: message.to_string(),
            });
        }
    }
    fields
}

fn build_token_config(jwt_settings: &AuthJwtSettings) -> Result<TokenConfig, AppError> {
    let algorithm = match jwt_settings.algorithm.to_uppercase().as_str() {
        "HS256" => Algorithm::HS256,
        "RS256" => Algorithm::RS256,
        other => {
            let _ = other;
            return Err(AppError::from(InternalError::Unknown));
        }
    };

    let (encoding_key, decoding_key) = match algorithm {
        Algorithm::HS256 => {
            let secret = jwt_settings
                .secret
                .clone()
                .ok_or_else(|| AppError::from(InternalError::Unknown))?;
            (
                EncodingKey::from_secret(secret.as_bytes()),
                DecodingKey::from_secret(secret.as_bytes()),
            )
        }
        Algorithm::RS256 => {
            let private = jwt_settings
                .private_key
                .clone()
                .ok_or_else(|| AppError::from(InternalError::Unknown))?;
            let public = jwt_settings
                .public_key
                .clone()
                .ok_or_else(|| AppError::from(InternalError::Unknown))?;
            (
                EncodingKey::from_rsa_pem(private.as_bytes()).map_err(|_| AppError::from(InternalError::Unknown))?,
                DecodingKey::from_rsa_pem(public.as_bytes()).map_err(|_| AppError::from(InternalError::Unknown))?,
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

fn map_token_error(err: TokenError) -> AppError {
    match err {
        TokenError::RefreshDisabled => AppError::from(BusinessError::Auth(AuthFlowError::RefreshDisabled)),
        TokenError::Decode(_) => AppError::from(BusinessError::Auth(AuthFlowError::TokenInvalid)),
        TokenError::Encode(_) => AppError::from(InternalError::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{AuthJwtSettings, AuthPasswordSettings, AuthSettings};
    use crate::domain::user::USERNAME_REGEX;
    use crate::domain::{HashedPassword, User};
    use crate::repository::user::{NewUser, UserRepository};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::RwLock;
    use chrono::Utc;

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

    fn service(repo: InMemoryUserRepository) -> AuthService<InMemoryUserRepository> {
        let settings = default_settings();
        AuthService::new(repo, &settings, &settings.jwt).unwrap()
    }

    #[tokio::test]
    async fn register_and_login_flow() {
        let repo = InMemoryUserRepository::default();
        let service = service(repo.clone());

        let register = RegisterRequest {
            username: "user123".into(),
            password: "password123".into(),
        };
        let profile = service.register(register).await.unwrap();
        assert_eq!(profile.username, "user123");

        let login = LoginRequest {
            username: "user123".into(),
            password: "password123".into(),
        };
        let tokens = service.login(login).await.unwrap();
        assert!(!tokens.access_token.is_empty());
    }

    #[tokio::test]
    async fn refresh_flow() {
        let repo = InMemoryUserRepository::default();
        let service = service(repo.clone());

        service
            .register(RegisterRequest {
                username: "user_refresh".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        let login_tokens = service
            .login(LoginRequest {
                username: "user_refresh".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();
        let refresh_token = login_tokens.refresh_token.expect("refresh token");

        let refreshed = service
            .refresh(RefreshRequest {
                refresh_token,
            })
            .await
            .unwrap();

        assert!(!refreshed.access_token.is_empty());
    }

    #[tokio::test]
    async fn profile_returns_user() {
        let repo = InMemoryUserRepository::default();
        let service = service(repo.clone());

        let profile = service
            .register(RegisterRequest {
                username: "profile_user".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();

        let fetched = service.profile(profile.id).await.unwrap();
        assert_eq!(fetched.username, "profile_user");
    }
}
