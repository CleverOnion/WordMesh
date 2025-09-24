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
}

#[derive(Debug, Error)]
pub enum BusinessError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Order(#[from] OrderError),
    #[error("Validation failed")] 
    Validation(Vec<ValidationField>),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")] 
    UserNotFound,
    #[error("Invalid username")] 
    InvalidUsername,
}

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("Order not found")] 
    OrderNotFound,
    #[error("Order already paid")] 
    OrderAlreadyPaid,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection failed")] 
    ConnectionFailed,
    #[error("Unique constraint violation")] 
    UniqueConstraintViolation,
}

#[derive(Debug, Error)]
pub enum ExternalError {
    #[error("HTTP client error")] 
    HttpClientError,
    #[error("Request timeout")] 
    Timeout,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Unauthorized")] 
    Unauthorized,
    #[error("Token expired")] 
    TokenExpired,
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Internal panic")] 
    Panic,
    #[error("Unknown error")] 
    Unknown,
}

#[derive(Debug, Serialize)]
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
                    let body = ApiResponse::error_with_trace(4001, message, trace_id);
                    HttpResponse::Ok().json(body)
                }
                _ => HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                    4000,
                    be.to_string(),
                    ResponseBuilder::current_trace_id(),
                )),
            },
            AppError::AuthError(ae) => HttpResponse::Ok().json(
                ApiResponse::<serde_json::Value>::error_with_trace(
                    4010,
                    ae.to_string(),
                    ResponseBuilder::current_trace_id(),
                ),
            ),
            AppError::DbError(_) | AppError::ExternalError(_) | AppError::InternalError(_) => {
                HttpResponse::Ok().json(ApiResponse::<serde_json::Value>::error_with_trace(
                    5000,
                    "内部服务错误",
                    ResponseBuilder::current_trace_id(),
                ))
            }
        }
    }
}



