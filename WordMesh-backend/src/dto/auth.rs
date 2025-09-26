use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(
        length(min = 3, max = 32, message = "用户名长度必须在 3 到 32 之间"),
        custom(function = "crate::domain::user::validate_username_format")
    )]
    pub username: String,
    #[validate(length(min = 8, message = "密码长度至少 8 位"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(
        length(min = 3, max = 32, message = "用户名长度必须在 3 到 32 之间"),
        custom(function = "crate::domain::user::validate_username_format")
    )]
    pub username: String,
    #[validate(length(min = 8, message = "密码长度至少 8 位"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 10, message = "refresh_token 长度不合法"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: i64,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_request_validates() {
        let req = RegisterRequest {
            username: "valid_user".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn register_request_rejects_short_username() {
        let req = RegisterRequest {
            username: "ab".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn login_request_rejects_invalid_characters() {
        let req = LoginRequest {
            username: "invalid-name".into(),
            password: "password123".into(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn refresh_request_requires_min_length() {
        let req = RefreshRequest {
            refresh_token: "short".into(),
        };
        assert!(req.validate().is_err());
    }
}
