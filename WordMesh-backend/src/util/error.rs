use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

use super::response::{ApiResponse, ResponseBuilder};

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    BusinessError(#[from] BusinessError),
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    ExternalError(#[from] ExternalError),
    #[error(transparent)]
    AuthError(#[from] AuthError),
    #[error(transparent)]
    InternalError(#[from] InternalError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum BusinessError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Order(#[from] OrderError),
    #[error(transparent)]
    Auth(#[from] AuthFlowError),
    #[error(transparent)]
    Word(#[from] WordError),
    #[error(transparent)]
    Link(#[from] LinkError),
    #[error("Validation failed")]
    Validation(Vec<ValidationField>),
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid username")]
    InvalidUsername,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum OrderError {
    #[error("Order not found")]
    OrderNotFound,
    #[error("Order already paid")]
    OrderAlreadyPaid,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AuthFlowError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token invalid")]
    TokenInvalid,
    #[error("Refresh token disabled")]
    RefreshDisabled,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum WordError {
    #[error("Word already exists in network")]
    AlreadyExists,
    #[error("Word not found in user network")]
    NotInNetwork,
    #[error("Sense text already exists")]
    SenseDuplicate,
    #[error("Primary sense conflict")]
    PrimaryConflict,
}

impl WordError {
    fn code(&self) -> i32 {
        match self {
            WordError::AlreadyExists => 4201,
            WordError::NotInNetwork => 4202,
            WordError::SenseDuplicate => 4203,
            WordError::PrimaryConflict => 4204,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum LinkError {
    #[error("Link already exists")]
    Exists,
    #[error("Self link is forbidden")]
    SelfForbidden,
    #[error("Link target not found")]
    TargetNotFound,
    #[error("Link type is invalid")]
    TypeInvalid,
    #[error("Link limit exceeded")]
    LimitExceeded,
}

impl LinkError {
    fn code(&self) -> i32 {
        match self {
            LinkError::Exists => 4301,
            LinkError::SelfForbidden => 4302,
            LinkError::TargetNotFound => 4303,
            LinkError::TypeInvalid => 4304,
            LinkError::LimitExceeded => 4305,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection failed")]
    ConnectionFailed,
    #[error("Unique constraint violation")]
    UniqueConstraintViolation,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ExternalError {
    #[error("HTTP client error")]
    HttpClientError,
    #[error("Request timeout")]
    Timeout,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Token expired")]
    TokenExpired,
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum InternalError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Internal panic")]
    Panic,
    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug, Serialize, Clone)]
pub struct ValidationField {
    pub field: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // 规则：统一返回 HTTP 200，通过业务 code 区分错误类型
        match self {
            AppError::BusinessError(be) => match be {
                BusinessError::Validation(fields) => {
                    let trace_id = crate::util::response::ResponseBuilder::current_trace_id();
                    let message = "参数校验失败".to_string();
                    let mut body: ApiResponse<Vec<ValidationField>> =
                        ApiResponse::error_with_trace(4001, message, trace_id);
                    body.data = Some(fields.clone());
                    HttpResponse::Ok().json(body)
                }
                BusinessError::Auth(auth_error) => {
                    let code = match auth_error {
                        AuthFlowError::InvalidCredentials => 4011,
                        AuthFlowError::TokenExpired => 4012,
                        AuthFlowError::TokenInvalid => 4013,
                        AuthFlowError::RefreshDisabled => 4014,
                    };
                    HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                        code,
                        auth_error.to_string(),
                        ResponseBuilder::current_trace_id(),
                    ))
                }
                BusinessError::Word(word_error) => {
                    HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                        word_error.code(),
                        word_error.to_string(),
                        ResponseBuilder::current_trace_id(),
                    ))
                }
                BusinessError::Link(link_error) => {
                    HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                        link_error.code(),
                        link_error.to_string(),
                        ResponseBuilder::current_trace_id(),
                    ))
                }
                _ => HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                    4000,
                    be.to_string(),
                    ResponseBuilder::current_trace_id(),
                )),
            },
            AppError::AuthError(ae) => {
                HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                    4010,
                    ae.to_string(),
                    ResponseBuilder::current_trace_id(),
                ))
            }
            AppError::DbError(_)
            | AppError::ExternalError(_)
            | AppError::InternalError(_)
            | AppError::IoError(_) => {
                HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                    5000,
                    "内部服务错误",
                    ResponseBuilder::current_trace_id(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::to_bytes;

    #[actix_rt::test]
    async fn business_auth_error_maps_to_expected_code() {
        let error = AppError::from(BusinessError::from(AuthFlowError::InvalidCredentials));
        let response = error.error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], 4011);
        assert_eq!(json["message"], "Invalid credentials");
        assert!(json["data"].is_null());
        assert!(json["traceId"].is_string());
        assert!(json["timestamp"].is_number());
    }

    #[actix_rt::test]
    async fn validation_error_returns_fields() {
        let fields = vec![ValidationField {
            field: "username".into(),
            message: "required".into(),
        }];
        let error = AppError::from(BusinessError::Validation(fields.clone()));
        let response = error.error_response();
        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], 4001);
        let data = json["data"].as_array().expect("data array");
        assert_eq!(data[0]["field"], "username");
        assert_eq!(data[0]["message"], "required");
        assert!(json["traceId"].is_string());
        assert!(json["timestamp"].is_number());
    }

    #[actix_rt::test]
    async fn word_error_returns_expected_payload() {
        let error = AppError::from(BusinessError::from(WordError::AlreadyExists));
        let response = error.error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], 4201);
        assert_eq!(json["message"], "Word already exists in network");
        assert!(json["data"].is_null());
        assert!(json["traceId"].is_string());
        assert!(json["timestamp"].is_number());
    }

    #[actix_rt::test]
    async fn link_error_returns_expected_payload() {
        let error = AppError::from(BusinessError::from(LinkError::SelfForbidden));
        let response = error.error_response();
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);

        let body = to_bytes(response.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], 4302);
        assert_eq!(json["message"], "Self link is forbidden");
        assert!(json["data"].is_null());
        assert!(json["traceId"].is_string());
        assert!(json["timestamp"].is_number());
    }
}
