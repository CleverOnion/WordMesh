use std::future::{Future, Ready, ready};
use std::sync::Arc;

use crate::util::error::{AppError, AuthFlowError, BusinessError};
use crate::util::token::{self, Claims, TokenConfig, TokenError};
use actix_web::body::{BoxBody, EitherBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::{Error, HttpMessage, HttpRequest};
use actix_web::{FromRequest, dev::Payload};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

/// Authentication middleware that validates Bearer access tokens and injects
/// authenticated user claims into the request extensions.
#[derive(Clone)]
pub struct AuthGuard {
    token_config: Arc<TokenConfig>,
}

impl AuthGuard {
    pub fn new(token_config: Arc<TokenConfig>) -> Self {
        Self { token_config }
    }

    fn token_config(&self) -> Arc<TokenConfig> {
        self.token_config.clone()
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthGuardMiddleware {
            service: Rc::new(RefCell::new(service)),
            token_config: self.token_config(),
        }))
    }
}

pub struct AuthGuardMiddleware<S> {
    service: Rc<RefCell<S>>,
    token_config: Arc<TokenConfig>,
}

impl<S, B> Service<ServiceRequest> for AuthGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }


    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let token_config = self.token_config.clone();

        match authenticate_request(req.request(), token_config.as_ref()) {
            Ok((user, claims)) => {
                req.extensions_mut().insert(AuthenticatedUser {
                    user_id: user,
                    scope: claims.scope.clone(),
                    request_id: claims.request_id.clone(),
                    claims,
                });
            }
            Err(err) => {
                let response = err.error_response().map_into_boxed_body();
                let service_response = req.into_response(response).map_into_right_body();
                return Box::pin(async move { Ok(service_response) });
            }
        }

        let fut = self.service.borrow_mut().call(req);
        Box::pin(async move {
            let response = fut.await?;
            Ok(response.map_into_left_body())
        })
    }
}

fn authenticate_request(
    req: &HttpRequest,
    token_config: &TokenConfig,
) -> Result<(i64, Claims), Error> {
    let bearer = extract_bearer_token(req)?;
    let claims = validate_access_token(token_config, bearer)?;
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| actix_web::Error::from(app_error(AuthFlowError::TokenInvalid)))?;
    Ok((user_id, claims))
}

fn extract_bearer_token(req: &HttpRequest) -> Result<&str, Error> {
    let header_value = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| actix_web::Error::from(app_error(AuthFlowError::InvalidCredentials)))?;

    let header_str = header_value
        .to_str()
        .map_err(|_| actix_web::Error::from(app_error(AuthFlowError::InvalidCredentials)))?;

    if let Some(token) = header_str.strip_prefix("Bearer ") {
        Ok(token)
    } else {
        Err(actix_web::Error::from(app_error(
            AuthFlowError::InvalidCredentials,
        )))
    }
}

fn validate_access_token(config: &TokenConfig, token: &str) -> Result<Claims, Error> {
    match token::validate_token(config, token) {
        Ok(claims) => Ok(claims),
        Err(TokenError::Decode(err)) => {
            let flow_error = if matches!(
                err.kind(),
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
            ) {
                AuthFlowError::TokenExpired
            } else {
                AuthFlowError::TokenInvalid
            };
            Err(actix_web::Error::from(app_error(flow_error)))
        }
        Err(TokenError::Encode(_)) => Err(actix_web::Error::from(app_error(
            AuthFlowError::TokenInvalid,
        ))),
        Err(TokenError::RefreshDisabled) => Err(actix_web::Error::from(app_error(
            AuthFlowError::TokenInvalid,
        ))),
    }
}

fn app_error(flow_error: AuthFlowError) -> AppError {
    AppError::from(BusinessError::Auth(flow_error))
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub scope: Option<String>,
    pub request_id: Option<String>,
    pub claims: Claims,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            ready(Ok(user.clone()))
        } else {
            ready(Err(actix_web::Error::from(app_error(
                AuthFlowError::InvalidCredentials,
            ))))
        }
    }
}
