use actix_web::HttpResponse;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub traceId: String,
    pub timestamp: i64,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success_with_trace(data: T, trace_id: String) -> Self {
        Self {
            code: 2000,
            message: "OK".to_string(),
            data: Some(data),
            traceId: trace_id,
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn error_with_trace(code: i32, message: impl Into<String>, trace_id: String) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
            traceId: trace_id,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

pub struct ResponseBuilder;

impl ResponseBuilder {
    /// 构建成功响应（HTTP 200），包含统一结构与 traceId、时间戳
    pub fn ok<T>(data: T) -> Result<HttpResponse, crate::util::AppError>
    where
        T: Serialize,
    {
        let trace_id = Self::current_trace_id();
        let body = ApiResponse::success_with_trace(data, trace_id);
        Ok(HttpResponse::Ok().json(body))
    }

    /// 构建失败响应（HTTP 200），使用业务 code 与消息，data 为空
    pub fn from_error(code: i32, message: impl Into<String>) -> Result<HttpResponse, crate::util::AppError> {
        let trace_id = Self::current_trace_id();
        let body = ApiResponse::<serde_json::Value>::error_with_trace(code, message, trace_id);
        Ok(HttpResponse::Ok().json(body))
    }

    /// 获取当前请求的 traceId：优先从 task-local 获取，否则生成 UUID
    pub(crate) fn current_trace_id() -> String {
        // 优先使用请求作用域中的 Request-Id
        if let Some(id) = REQUEST_ID.try_with(|id| id.clone()).ok() {
            return id;
        }
        Uuid::new_v4().to_string()
    }
}

#[derive(Serialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Serialize)]
pub struct PagedData<T>
where
    T: Serialize,
{
    pub items: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Serialize)]
pub struct ValidationErrorData {
    pub field: String,
    pub message: String,
}

// 请求作用域的 Request-Id，用于响应与日志关联
tokio::task_local! {
    pub static REQUEST_ID: String;
}


