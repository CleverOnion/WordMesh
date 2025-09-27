use std::sync::Arc;

use tracing::instrument;

use crate::domain::word::UserSense;
use crate::repository::graph::GraphRepository;
use crate::repository::word::{NewUserSense, SenseUpdate, WordRepository};
use crate::service::word::{
    SenseInput, build_new_sense_payload, map_graph_error, map_user_sense_error,
    map_validation_error, map_word_error,
};
use crate::util::error::{AppError, BusinessError, WordError};
use crate::util::validation::{validate_non_empty_text, validate_note};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SenseUpdateInput {
    pub text: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
    pub note: Option<Option<String>>,
}

#[allow(dead_code)]
pub struct SenseService<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    word_repository: Arc<W>,
    graph_repository: Arc<G>,
}

impl<W, G> SenseService<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    #[allow(dead_code)]
    pub fn new(word_repository: W, graph_repository: G) -> Self {
        Self {
            word_repository: Arc::new(word_repository),
            graph_repository: Arc::new(graph_repository),
        }
    }

    #[allow(dead_code)]
    #[instrument(skip(self, input), fields(user_id = user_id, user_word_id = user_word_id))]
    pub async fn add_sense(
        &self,
        user_id: i64,
        user_word_id: i64,
        input: SenseInput,
    ) -> Result<UserSense, AppError> {
        self.word_repository
            .find_user_word(user_id, user_word_id)
            .await
            .map_err(map_word_error)?
            .ok_or_else(|| AppError::from(BusinessError::Word(WordError::NotInNetwork)))?;

        let new_sense = build_new_sense_payload(user_word_id, input)?;
        let created = self
            .word_repository
            .add_user_sense(new_sense)
            .await
            .map_err(map_word_error)?;

        if let Some(sense_id) = created.id() {
            self.graph_repository
                .upsert_node_sense(sense_id, user_id)
                .await
                .map_err(map_graph_error)?;
        }

        Ok(created)
    }

    #[allow(dead_code)]
    #[instrument(skip(self, input), fields(user_id = user_id, sense_id = sense_id))]
    pub async fn update_sense(
        &self,
        user_id: i64,
        sense_id: i64,
        input: SenseUpdateInput,
    ) -> Result<UserSense, AppError> {
        let update = build_sense_update(input)?;
        let updated = self
            .word_repository
            .update_user_sense(user_id, sense_id, update)
            .await
            .map_err(map_word_error)?;

        Ok(updated)
    }

    #[allow(dead_code)]
    #[instrument(skip(self), fields(user_id = user_id, sense_id = sense_id))]
    pub async fn remove_sense(&self, user_id: i64, sense_id: i64) -> Result<UserSense, AppError> {
        let removed = self
            .word_repository
            .remove_user_sense(user_id, sense_id)
            .await
            .map_err(map_word_error)?;

        if let Some(id) = removed.id() {
            self.graph_repository
                .remove_links_for_sense(id)
                .await
                .map_err(map_graph_error)?;
        }

        Ok(removed)
    }
}

fn build_sense_update(input: SenseUpdateInput) -> Result<SenseUpdate, AppError> {
    let mut update = SenseUpdate {
        text: None,
        is_primary: input.is_primary,
        sort_order: input.sort_order,
        note: None,
    };

    if let Some(text) = input.text {
        let value =
            validate_non_empty_text(text).map_err(|err| map_validation_error("sense.text", err))?;
        update.text = Some(value);
    }

    if let Some(note_option) = input.note {
        let value =
            validate_note(note_option).map_err(|err| map_validation_error("sense.note", err))?;
        update.note = Some(value);
    }

    Ok(update)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::word::{CanonicalKey, UserWord, UserWordError};
    use crate::repository::graph::{
        GraphRepository, GraphResult, SenseLinkFilter, SenseWordLinkKind, WordLinkFilter,
        WordLinkKind,
    };
    use crate::repository::word::{
        SearchParams, SearchScope, UpsertUserWord, UserWordAggregate, WordRecord, WordRepository,
        WordRepositoryError,
    };
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};

    struct StubWordRepository {
        user_word: Option<UserWordAggregate>,
    }

    impl StubWordRepository {
        fn with_existing_word() -> Self {
            let aggregate = UserWordAggregate {
                word: WordRecord {
                    id: 10,
                    text: "hello".into(),
                    canonical_key: CanonicalKey::new("hello").unwrap(),
                    created_at: Utc::now(),
                },
                user_word: UserWord::from_parts(None, 1, 10, vec![], None, vec![], Utc::now())
                    .unwrap(),
            };
            Self {
                user_word: Some(aggregate),
            }
        }
    }

    #[async_trait]
    impl WordRepository for StubWordRepository {
        async fn upsert_word(
            &self,
            _canonical: &CanonicalKey,
            _text: &str,
        ) -> Result<WordRecord, WordRepositoryError> {
            unimplemented!()
        }

        async fn upsert_user_word(
            &self,
            _payload: UpsertUserWord,
        ) -> Result<UserWordAggregate, WordRepositoryError> {
            unimplemented!()
        }

        async fn find_user_word(
            &self,
            _user_id: i64,
            _user_word_id: i64,
        ) -> Result<Option<UserWordAggregate>, WordRepositoryError> {
            Ok(self.user_word.clone())
        }

        async fn remove_user_word(
            &self,
            _user_id: i64,
            _user_word_id: i64,
        ) -> Result<(), WordRepositoryError> {
            unimplemented!()
        }

        async fn add_user_sense(
            &self,
            _sense: NewUserSense,
        ) -> Result<UserSense, WordRepositoryError> {
            let sense = UserSense::new("meaning", true, 0, None).unwrap();
            Ok(sense)
        }

        async fn update_user_sense(
            &self,
            _user_id: i64,
            _sense_id: i64,
            _update: SenseUpdate,
        ) -> Result<UserSense, WordRepositoryError> {
            let mut sense = UserSense::new("meaning", true, 0, None).unwrap();
            sense.set_text("updated").unwrap();
            Ok(sense)
        }

        async fn remove_user_sense(
            &self,
            _user_id: i64,
            _sense_id: i64,
        ) -> Result<UserSense, WordRepositoryError> {
            Err(WordRepositoryError::UserSense(
                crate::domain::word::UserSenseError::InvalidNote,
            ))
        }

        async fn search(
            &self,
            _params: SearchParams,
        ) -> Result<Vec<UserWordAggregate>, WordRepositoryError> {
            Ok(vec![])
        }
    }

    struct StubGraphRepository;

    #[async_trait]
    impl GraphRepository for StubGraphRepository {
        async fn create_word_link(
            &self,
            _user_id: i64,
            _word_a_id: i64,
            _word_b_id: i64,
            _kind: WordLinkKind,
            _note: Option<String>,
        ) -> GraphResult<crate::repository::graph::WordLinkRecord> {
            unimplemented!()
        }

        async fn delete_word_link(
            &self,
            _user_id: i64,
            _word_a_id: i64,
            _word_b_id: i64,
            _kind: WordLinkKind,
        ) -> GraphResult<()> {
            Ok(())
        }

        async fn list_word_links(
            &self,
            _filter: WordLinkFilter,
        ) -> GraphResult<Vec<crate::repository::graph::WordLinkRecord>> {
            Ok(vec![])
        }

        async fn create_sense_word_link(
            &self,
            _user_id: i64,
            _sense_id: i64,
            _source_word_id: i64,
            _target_word_id: i64,
            _kind: SenseWordLinkKind,
            _note: Option<String>,
        ) -> GraphResult<crate::repository::graph::SenseWordLinkRecord> {
            unimplemented!()
        }

        async fn delete_sense_word_link(
            &self,
            _user_id: i64,
            _sense_id: i64,
            _target_word_id: i64,
            _kind: SenseWordLinkKind,
        ) -> GraphResult<()> {
            Ok(())
        }

        async fn list_sense_word_links(
            &self,
            _filter: SenseLinkFilter,
        ) -> GraphResult<Vec<crate::repository::graph::SenseWordLinkRecord>> {
            Ok(vec![])
        }

        async fn remove_links_for_sense(&self, _sense_id: i64) -> GraphResult<()> {
            Ok(())
        }

        async fn upsert_node_word(&self, _word_id: i64) -> GraphResult<()> {
            Ok(())
        }

        async fn upsert_node_sense(&self, _sense_id: i64, _user_id: i64) -> GraphResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn add_sense_success() {
        let service = SenseService::new(
            StubWordRepository::with_existing_word(),
            StubGraphRepository,
        );
        let sense = service
            .add_sense(
                1,
                10,
                SenseInput {
                    text: "meaning".into(),
                    is_primary: true,
                    sort_order: 0,
                    note: None,
                },
            )
            .await
            .expect("sense added");
        assert_eq!(sense.text(), "meaning");
    }

    #[tokio::test]
    async fn remove_sense_maps_error() {
        let service = SenseService::new(
            StubWordRepository::with_existing_word(),
            StubGraphRepository,
        );
        let err = service.remove_sense(1, 99).await.unwrap_err();
        assert!(matches!(
            err,
            AppError::BusinessError(BusinessError::Validation(_))
        ));
    }
}
