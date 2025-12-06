use actix_web::{HttpResponse, web};
use std::sync::Arc;
use tracing::instrument;
use validator::Validate;

use crate::dto::word::{AddSenseRequest, UpdateSenseRequest};
use crate::middleware::{AuthGuard, AuthenticatedUser};
use crate::service::sense::SenseService;
use crate::repository::word::WordRepository;
use crate::repository::graph::GraphRepository;
use crate::util::{AppError, ResponseBuilder};

#[derive(Clone)]
pub struct SenseController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    service: Arc<SenseService<W, G>>,
    auth_guard: AuthGuard,
}

impl<W, G> SenseController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    pub fn new(service: SenseService<W, G>, auth_guard: AuthGuard) -> Self {
        Self {
            service: Arc::new(service),
            auth_guard,
        }
    }

    pub fn configure(cfg: &mut web::ServiceConfig, controller: web::Data<SenseController<W, G>>) {
        let guard = controller.auth_guard.clone();
        cfg.service(
            web::scope("/words")
                .app_data(controller.clone())
                .service(
                    web::scope("/my")
                        .wrap(guard)
                        .service(
                            web::resource("/{user_word_id}/senses")
                                .route(web::post().to(Self::add_sense)),
                        )
                        .service(
                            web::resource("/senses/{sense_id}")
                                .route(web::patch().to(Self::update_sense))
                                .route(web::delete().to(Self::remove_sense)),
                        ),
                ),
        );
    }

    #[instrument(skip(controller, payload, identity, path))]
    async fn add_sense(
        controller: web::Data<SenseController<W, G>>,
        path: web::Path<i64>,
        payload: web::Json<AddSenseRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let user_word_id = path.into_inner();
        
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let input = request.into();
        let result = controller
            .service
            .add_sense(identity.user_id, user_word_id, input)
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, payload, identity, path))]
    async fn update_sense(
        controller: web::Data<SenseController<W, G>>,
        path: web::Path<i64>,
        payload: web::Json<UpdateSenseRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let sense_id = path.into_inner();
        
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let input = request.into();
        let result = controller
            .service
            .update_sense(identity.user_id, sense_id, input)
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, identity, path))]
    async fn remove_sense(
        controller: web::Data<SenseController<W, G>>,
        path: web::Path<i64>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let sense_id = path.into_inner();
        
        let result = controller
            .service
            .remove_sense(identity.user_id, sense_id)
            .await?;
        ResponseBuilder::ok(result)
    }
}

fn validation_errors(err: validator::ValidationErrors) -> Vec<crate::util::error::ValidationField> {
    let mut fields = Vec::new();
    for (field, errors) in err.field_errors() {
        for error in errors {
            let message = error.message.clone().unwrap_or_else(|| "参数错误".into());
            fields.push(crate::util::error::ValidationField {
                field: field.to_string(),
                message: message.to_string(),
            });
        }
    }
    fields
}





