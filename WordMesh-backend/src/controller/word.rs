use actix_web::{HttpResponse, web};
use std::sync::Arc;
use tracing::instrument;
use validator::Validate;

use crate::dto::word::{AddWordRequest, SearchRequest, AddSenseRequest, UpdateSenseRequest};
use crate::middleware::{AuthGuard, AuthenticatedUser};
use crate::service::word::WordService;
use crate::service::sense::SenseService;
use crate::repository::word::WordRepository;
use crate::repository::graph::GraphRepository;
use crate::util::{AppError, ResponseBuilder};

#[derive(Clone)]
pub struct WordController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    word_service: Arc<WordService<W, G>>,
    sense_service: Arc<SenseService<W, G>>,
    auth_guard: AuthGuard,
}

impl<W, G> WordController<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    pub fn new(word_service: WordService<W, G>, sense_service: SenseService<W, G>, auth_guard: AuthGuard) -> Self {
        Self {
            word_service: Arc::new(word_service),
            sense_service: Arc::new(sense_service),
            auth_guard,
        }
    }

    pub fn configure(cfg: &mut web::ServiceConfig, controller: web::Data<WordController<W, G>>) {
        let guard = controller.auth_guard.clone();
        cfg.service(
            web::scope("/words")
                .app_data(controller.clone())
                .service(
                    web::scope("/my")
                        .wrap(guard)
                        // 字面量路径必须优先注册
                        .route("", web::post().to(Self::add_to_my_network))
                        .route("/search", web::get().to(Self::search_my_network))
                        // 更具体的路径优先
                        .route("/{user_word_id}/senses", web::post().to(Self::add_sense))
                        .route("/senses/{sense_id}", web::patch().to(Self::update_sense))
                        .route("/senses/{sense_id}", web::delete().to(Self::remove_sense))
                        // 参数路径最后注册
                        .route("/{user_word_id}", web::delete().to(Self::remove_from_my_network)),
                ),
        );
    }

    #[instrument(skip(controller, payload, identity))]
    async fn add_to_my_network(
        controller: web::Data<WordController<W, G>>,
        payload: web::Json<AddWordRequest>,
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

        let input = request.into();
        let result = controller
            .word_service
            .add_to_my_network(identity.user_id, input)
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, query, identity))]
    async fn search_my_network(
        controller: web::Data<WordController<W, G>>,
        query: web::Query<SearchRequest>,
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

        let options = request.into_options();
        let results = controller
            .word_service
            .search_in_my_network(identity.user_id, options)
            .await?;
        ResponseBuilder::ok(results)
    }

    #[instrument(skip(controller, identity, path))]
    async fn remove_from_my_network(
        controller: web::Data<WordController<W, G>>,
        path: web::Path<i64>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let user_word_id = path.into_inner();
        
        controller
            .word_service
            .remove_from_my_network(identity.user_id, user_word_id)
            .await?;
        
        ResponseBuilder::ok(())
    }

    // Sense 相关路由处理函数
    #[instrument(skip(controller, payload, identity, path))]
    async fn add_sense(
        controller: web::Data<WordController<W, G>>,
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
            .sense_service
            .add_sense(identity.user_id, user_word_id, input)
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, payload, identity, path))]
    async fn update_sense(
        controller: web::Data<WordController<W, G>>,
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
            .sense_service
            .update_sense(identity.user_id, sense_id, input)
            .await?;
        ResponseBuilder::ok(result)
    }

    #[instrument(skip(controller, identity, path))]
    async fn remove_sense(
        controller: web::Data<WordController<W, G>>,
        path: web::Path<i64>,
        identity: AuthenticatedUser,
    ) -> Result<HttpResponse, AppError> {
        let sense_id = path.into_inner();
        
        let result = controller
            .sense_service
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

