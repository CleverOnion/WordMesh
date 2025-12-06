use actix_web::{HttpResponse, web};
use std::sync::Arc;
use tracing::instrument;
use validator::Validate;

use crate::dto::word::{
    CreateSenseWordLinkRequest, CreateWordLinkRequest, DeleteSenseWordLinkRequest,
    DeleteWordLinkRequest, LinkEndpointDto, LinkQuery,
};
use crate::middleware::{AuthGuard, AuthenticatedUser};
use crate::repository::graph::{SenseLinkFilter, SenseWordLinkKind, WordLinkFilter, WordLinkKind};
use crate::service::assoc::AssocService;
use crate::repository::word::WordRepository;
use crate::repository::graph::GraphRepository;
use crate::util::{AppError, ResponseBuilder};

#[derive(Clone)]
pub struct AssocController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    service: Arc<AssocService<W, G>>,
    auth_guard: AuthGuard,
}

impl<W, G> AssocController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    pub fn new(service: AssocService<W, G>, auth_guard: AuthGuard) -> Self {
        Self {
            service: Arc::new(service),
            auth_guard,
        }
    }

    pub fn configure(cfg: &mut web::ServiceConfig, controller: web::Data<AssocController<W, G>>) {
        let guard = controller.auth_guard.clone();
        cfg.service(
            web::scope("/words")
                .app_data(controller.clone())
                .service(
                    web::scope("/associations")
                        .wrap(guard)
                        .service(
                            web::resource("/word")
                                .route(web::post().to(Self::create_word_link))
                                .route(web::delete().to(Self::delete_word_link)),
                        )
                        .service(
                            web::resource("/sense-word")
                                .route(web::post().to(Self::create_sense_word_link))
                                .route(web::delete().to(Self::delete_sense_word_link)),
                        )
                        .route("", web::get().to(Self::list_links)),
                ),
        );
    }

    #[instrument(skip(controller, payload, identity))]
    async fn create_word_link(
        controller: web::Data<AssocController<W, G>>,
        payload: web::Json<CreateWordLinkRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let kind: WordLinkKind = request.kind.into();
        let result = controller
            .service
            .create_word_link(
                identity.user_id,
                request.word_a_id,
                request.word_b_id,
                kind,
                request.note,
            )
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, payload, identity))]
    async fn create_sense_word_link(
        controller: web::Data<AssocController<W, G>>,
        payload: web::Json<CreateSenseWordLinkRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let kind: SenseWordLinkKind = request.kind.into();

        let result = controller
            .service
            .create_sense_word_link(
                identity.user_id,
                request.sense_id,
                request.target_word_id,
                kind,
                request.note,
            )
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, query, identity))]
    async fn list_links(
        controller: web::Data<AssocController<W, G>>,
        query: web::Query<LinkQuery>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        // Validate input
        let request = query.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        match request.endpoint_type {
            LinkEndpointDto::Word => {
                let kind = request
                    .kind
                    .as_ref()
                    .and_then(|k| parse_word_link_kind(k));
                let filter = WordLinkFilter {
                    user_id: identity.user_id,
                    word_id: request.endpoint_id,
                    kind,
                    limit: request.limit,
                    offset: request.offset,
                };
                let results = controller.service.list_word_links(filter).await?;
                ResponseBuilder::ok(results)
            }
            LinkEndpointDto::Sense => {
                let kind = request
                    .kind
                    .as_ref()
                    .and_then(|k| parse_sense_word_link_kind(k));
                let filter = SenseLinkFilter {
                    user_id: identity.user_id,
                    sense_id: request.endpoint_id,
                    kind,
                    limit: request.limit,
                    offset: request.offset,
                };
                let results = controller.service.list_sense_word_links(filter).await?;
                ResponseBuilder::ok(results)
            }
        }
    }

    #[instrument(skip(controller, payload, identity))]
    async fn delete_word_link(
        controller: web::Data<AssocController<W, G>>,
        payload: web::Json<DeleteWordLinkRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let kind: WordLinkKind = request.kind.into();
        controller
            .service
            .delete_word_link(
                identity.user_id,
                request.word_a_id,
                request.word_b_id,
                kind,
            )
            .await?;
        ResponseBuilder::ok(())
    }

    #[instrument(skip(controller, payload, identity))]
    async fn delete_sense_word_link(
        controller: web::Data<AssocController<W, G>>,
        payload: web::Json<DeleteSenseWordLinkRequest>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        // Validate input
        let request = payload.into_inner();
        request
            .validate()
            .map_err(|err| {
                AppError::from(crate::util::error::BusinessError::Validation(
                    validation_errors(err),
                ))
            })?;

        let kind: SenseWordLinkKind = request.kind.into();
        controller
            .service
            .delete_sense_word_link(
                identity.user_id,
                request.sense_id,
                request.target_word_id,
                kind,
            )
            .await?;
        ResponseBuilder::ok(())
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

fn parse_word_link_kind(s: &str) -> Option<WordLinkKind> {
    match s {
        "similar_form" => Some(WordLinkKind::SimilarForm),
        "root_affix" => Some(WordLinkKind::RootAffix),
        _ => None,
    }
}

fn parse_sense_word_link_kind(s: &str) -> Option<SenseWordLinkKind> {
    match s {
        "synonym" => Some(SenseWordLinkKind::Synonym),
        "antonym" => Some(SenseWordLinkKind::Antonym),
        "related" => Some(SenseWordLinkKind::Related),
        _ => None,
    }
}


