use std::future::{Ready, ready};
use std::pin::Pin;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use uuid::Uuid;

/// 确保每个请求拥有 `X-Request-Id` 的中间件：
/// - 若请求头包含 `X-Request-Id` 则复用，否则生成 UUID v4
/// - 将 Request-Id 写入请求扩展（extensions）便于下游读取
/// - 将 `X-Request-Id` 写入响应头，便于前后端与日志关联
/// - 通过 task-local 传递 Request-Id，便于响应构建等任意位置读取
pub struct RequestId;

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddleware { service }))
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let header_name = HeaderName::from_static("x-request-id");
        let incoming = req
            .headers()
            .get(&header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // 写入请求扩展，便于处理函数直接读取
        req.extensions_mut().insert(incoming.clone());

        let fut = self.service.call(req);

        // 在带有 Request-Id 的 task-local 作用域下执行下游服务
        Box::pin(async move {
            let result = crate::util::response::REQUEST_ID
                .scope(incoming.clone(), async move { fut.await })
                .await;

            match result {
                Ok(mut res) => {
                    // 确保在响应头中设置 `X-Request-Id`
                    if let Ok(val) = HeaderValue::from_str(&incoming) {
                        res.headers_mut().insert(header_name, val);
                    }
                    Ok(res)
                }
                Err(e) => Err(e),
            }
        })
    }
}
