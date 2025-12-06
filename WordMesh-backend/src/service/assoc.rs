use std::sync::Arc;

use tracing::instrument;

use crate::repository::graph::{
    GraphRepository, SenseLinkFilter, SenseWordLinkKind, WordLinkFilter,
    WordLinkKind, WordLinkRecord, SenseWordLinkRecord,
};
use crate::repository::word::{WordRepository, UpsertUserWord};
use crate::domain::word::CanonicalKey;
use crate::util::error::{AppError, BusinessError, LinkError};
use crate::service::word::map_graph_error;

#[allow(dead_code)]
pub struct AssocService<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    word_repository: Arc<W>,
    graph_repository: Arc<G>,
}

impl<W, G> AssocService<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    pub fn new(word_repository: W, graph_repository: G) -> Self {
        Self {
            word_repository: Arc::new(word_repository),
            graph_repository: Arc::new(graph_repository),
        }
    }

    #[allow(dead_code)]
    #[instrument(skip(self), fields(user_id = user_id))]
    pub async fn create_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
        note: Option<String>,
    ) -> Result<WordLinkRecord, AppError> {
        // Check self-link
        if word_a_id == word_b_id {
            return Err(AppError::from(BusinessError::Link(LinkError::SelfForbidden)));
        }

        // Verify both words are in user's network
        // Note: This check should be done at controller level or here
        // For now, we'll rely on the repository to handle this

        self.graph_repository
            .create_word_link(user_id, word_a_id, word_b_id, kind, note)
            .await
            .map_err(map_graph_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self), fields(user_id = user_id))]
    pub async fn create_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
        note: Option<String>,
    ) -> Result<SenseWordLinkRecord, AppError> {
        // Get sense and its associated word_id
        let (_, source_word_id) = self
            .word_repository
            .find_sense_by_id(user_id, sense_id)
            .await
            .map_err(|_| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?
            .ok_or_else(|| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?;

        // Check self-link
        if source_word_id == target_word_id {
            return Err(AppError::from(BusinessError::Link(LinkError::SelfForbidden)));
        }

        // Get target word to ensure it exists and get its text
        let target_word = self
            .word_repository
            .find_word_by_id(target_word_id)
            .await
            .map_err(|_| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?
            .ok_or_else(|| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?;

        // Ensure target word is in user's network (idempotent operation)
        let canonical_key = CanonicalKey::new(&target_word.text)
            .map_err(|_| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?;
        
        let _ = self
            .word_repository
            .upsert_user_word(UpsertUserWord {
                user_id,
                word_text: target_word.text,
                canonical_key,
                tags: vec![],
                note: None,
            })
            .await
            .map_err(|_| AppError::from(BusinessError::Link(LinkError::TargetNotFound)))?;

        // Create the link
        self.graph_repository
            .create_sense_word_link(
                user_id,
                sense_id,
                source_word_id,
                target_word_id,
                kind,
                note,
            )
            .await
            .map_err(map_graph_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn list_word_links(
        &self,
        filter: WordLinkFilter,
    ) -> Result<Vec<WordLinkRecord>, AppError> {
        self.graph_repository
            .list_word_links(filter)
            .await
            .map_err(map_graph_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn list_sense_word_links(
        &self,
        filter: SenseLinkFilter,
    ) -> Result<Vec<SenseWordLinkRecord>, AppError> {
        self.graph_repository
            .list_sense_word_links(filter)
            .await
            .map_err(map_graph_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn delete_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
    ) -> Result<(), AppError> {
        self.graph_repository
            .delete_word_link(user_id, word_a_id, word_b_id, kind)
            .await
            .map_err(map_graph_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn delete_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
    ) -> Result<(), AppError> {
        self.graph_repository
            .delete_sense_word_link(user_id, sense_id, target_word_id, kind)
            .await
            .map_err(map_graph_error)
    }
}

