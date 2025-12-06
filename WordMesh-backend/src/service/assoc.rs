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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use crate::domain::word::{UserSense, UserWord};
    use crate::repository::graph::{GraphRepository, GraphResult, SenseLinkFilter, SenseWordLinkKind, WordLinkFilter, WordLinkKind};
    use crate::repository::word::{NewUserSense, SearchParams, SearchScope, SenseUpdate, UpsertUserWord, UserWordAggregate, WordRecord, WordRepository, WordRepositoryError};

    struct StubWordRepository {
        sense: Option<(UserSense, i64)>,
        word: Option<WordRecord>,
        upsert_success: bool,
    }

    impl StubWordRepository {
        fn with_sense_and_word() -> Self {
            let sense = UserSense::from_parts(
                Some(1),
                "meaning".to_string(),
                true,
                0,
                None,
                Utc::now(),
            ).unwrap();
            let word = WordRecord {
                id: 10,
                text: "hello".into(),
                canonical_key: CanonicalKey::new("hello").unwrap(),
                created_at: Utc::now(),
            };
            Self {
                sense: Some((sense, 10)),
                word: Some(word),
                upsert_success: true,
            }
        }

        fn without_sense() -> Self {
            Self {
                sense: None,
                word: Some(WordRecord {
                    id: 10,
                    text: "hello".into(),
                    canonical_key: CanonicalKey::new("hello").unwrap(),
                    created_at: Utc::now(),
                }),
                upsert_success: true,
            }
        }

        fn without_word() -> Self {
            Self {
                sense: Some((
                    UserSense::from_parts(
                        Some(1),
                        "meaning".to_string(),
                        true,
                        0,
                        None,
                        Utc::now(),
                    ).unwrap(),
                    10,
                )),
                word: None,
                upsert_success: true,
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
            if self.upsert_success {
                let word = WordRecord {
                    id: 10,
                    text: "target".into(),
                    canonical_key: CanonicalKey::new("target").unwrap(),
                    created_at: Utc::now(),
                };
                let user_word = UserWord::from_parts(None, 1, 10, vec![], None, vec![], Utc::now())
                    .unwrap();
                Ok(UserWordAggregate { word, user_word })
            } else {
                Err(WordRepositoryError::UserWord(
                    crate::domain::word::UserWordError::InvalidNote,
                ))
            }
        }

        async fn find_user_word(
            &self,
            _user_id: i64,
            _user_word_id: i64,
        ) -> Result<Option<UserWordAggregate>, WordRepositoryError> {
            Ok(None)
        }

        async fn remove_user_word(
            &self,
            _user_id: i64,
            _user_word_id: i64,
        ) -> Result<(), WordRepositoryError> {
            Ok(())
        }

        async fn add_user_sense(
            &self,
            _sense: NewUserSense,
        ) -> Result<UserSense, WordRepositoryError> {
            unimplemented!()
        }

        async fn update_user_sense(
            &self,
            _user_id: i64,
            _sense_id: i64,
            _update: SenseUpdate,
        ) -> Result<UserSense, WordRepositoryError> {
            unimplemented!()
        }

        async fn remove_user_sense(
            &self,
            _user_id: i64,
            _sense_id: i64,
        ) -> Result<UserSense, WordRepositoryError> {
            unimplemented!()
        }

        async fn find_sense_by_id(
            &self,
            _user_id: i64,
            _sense_id: i64,
        ) -> Result<Option<(UserSense, i64)>, WordRepositoryError> {
            Ok(self.sense.clone())
        }

        async fn find_word_by_id(
            &self,
            _word_id: i64,
        ) -> Result<Option<WordRecord>, WordRepositoryError> {
            Ok(self.word.clone())
        }

        async fn search(
            &self,
            _params: SearchParams,
        ) -> Result<Vec<UserWordAggregate>, WordRepositoryError> {
            Ok(vec![])
        }
    }

    struct StubGraphRepository {
        create_word_link_result: Option<Result<WordLinkRecord, crate::repository::graph::GraphRepositoryError>>,
        create_sense_word_link_result: Option<Result<SenseWordLinkRecord, crate::repository::graph::GraphRepositoryError>>,
        list_word_links_result: Vec<WordLinkRecord>,
        list_sense_word_links_result: Vec<SenseWordLinkRecord>,
    }

    impl StubGraphRepository {
        fn success() -> Self {
            Self {
                create_word_link_result: Some(Ok(WordLinkRecord {
                    link_id: "link-1".into(),
                    user_id: 1,
                    kind: WordLinkKind::SimilarForm,
                    note: None,
                    created_at: Utc::now(),
                    word_a_id: 10,
                    word_b_id: 20,
                })),
                create_sense_word_link_result: Some(Ok(SenseWordLinkRecord {
                    link_id: "link-1".into(),
                    user_id: 1,
                    kind: SenseWordLinkKind::Synonym,
                    note: None,
                    created_at: Utc::now(),
                    sense_id: 1,
                    source_word_id: 10,
                    target_word_id: 20,
                })),
                list_word_links_result: vec![],
                list_sense_word_links_result: vec![],
            }
        }

        fn with_self_link_error() -> Self {
            Self {
                create_word_link_result: Some(Err(
                    crate::repository::graph::GraphRepositoryError::Business(
                        BusinessError::Link(LinkError::SelfForbidden),
                    ),
                )),
                create_sense_word_link_result: Some(Err(
                    crate::repository::graph::GraphRepositoryError::Business(
                        BusinessError::Link(LinkError::SelfForbidden),
                    ),
                )),
                list_word_links_result: vec![],
                list_sense_word_links_result: vec![],
            }
        }
    }

    #[async_trait]
    impl GraphRepository for StubGraphRepository {
        async fn create_word_link(
            &self,
            _user_id: i64,
            _word_a_id: i64,
            _word_b_id: i64,
            _kind: WordLinkKind,
            _note: Option<String>,
        ) -> GraphResult<WordLinkRecord> {
            if let Some(result) = &self.create_word_link_result {
                if let Ok(link) = result {
                    Ok(WordLinkRecord {
                        link_id: link.link_id.clone(),
                        user_id: link.user_id,
                        kind: link.kind,
                        note: link.note.clone(),
                        created_at: link.created_at,
                        word_a_id: link.word_a_id,
                        word_b_id: link.word_b_id,
                    })
                } else {
                    // Return a test error
                    Err(crate::repository::graph::GraphRepositoryError::Business(
                        BusinessError::Link(LinkError::SelfForbidden),
                    ))
                }
            } else {
                Ok(WordLinkRecord {
                    link_id: "link-1".into(),
                    user_id: 1,
                    kind: WordLinkKind::SimilarForm,
                    note: None,
                    created_at: Utc::now(),
                    word_a_id: 10,
                    word_b_id: 20,
                })
            }
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
        ) -> GraphResult<Vec<WordLinkRecord>> {
            Ok(self.list_word_links_result.clone())
        }

        async fn create_sense_word_link(
            &self,
            _user_id: i64,
            _sense_id: i64,
            _source_word_id: i64,
            _target_word_id: i64,
            _kind: SenseWordLinkKind,
            _note: Option<String>,
        ) -> GraphResult<SenseWordLinkRecord> {
            if let Some(result) = &self.create_sense_word_link_result {
                if let Ok(link) = result {
                    Ok(SenseWordLinkRecord {
                        link_id: link.link_id.clone(),
                        user_id: link.user_id,
                        kind: link.kind,
                        note: link.note.clone(),
                        created_at: link.created_at,
                        sense_id: link.sense_id,
                        source_word_id: link.source_word_id,
                        target_word_id: link.target_word_id,
                    })
                } else {
                    // Return a test error
                    Err(crate::repository::graph::GraphRepositoryError::Business(
                        BusinessError::Link(LinkError::SelfForbidden),
                    ))
                }
            } else {
                Ok(SenseWordLinkRecord {
                    link_id: "link-1".into(),
                    user_id: 1,
                    kind: SenseWordLinkKind::Synonym,
                    note: None,
                    created_at: Utc::now(),
                    sense_id: 1,
                    source_word_id: 10,
                    target_word_id: 20,
                })
            }
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
        ) -> GraphResult<Vec<SenseWordLinkRecord>> {
            Ok(self.list_sense_word_links_result.clone())
        }

        async fn remove_links_for_sense(
            &self,
            _sense_id: i64,
        ) -> GraphResult<()> {
            Ok(())
        }

        async fn upsert_node_word(
            &self,
            _word_id: i64,
        ) -> GraphResult<()> {
            Ok(())
        }

        async fn upsert_node_sense(
            &self,
            _sense_id: i64,
            _user_id: i64,
        ) -> GraphResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_word_link_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .create_word_link(1, 10, 20, WordLinkKind::SimilarForm, None)
            .await;
        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.word_a_id, 10);
        assert_eq!(link.word_b_id, 20);
    }

    #[tokio::test]
    async fn create_word_link_self_link_error() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let err = service
            .create_word_link(1, 10, 10, WordLinkKind::SimilarForm, None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::BusinessError(BusinessError::Link(LinkError::SelfForbidden))
        ));
    }

    #[tokio::test]
    async fn create_sense_word_link_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .create_sense_word_link(1, 1, 20, SenseWordLinkKind::Synonym, None)
            .await;
        assert!(result.is_ok());
        let link = result.unwrap();
        assert_eq!(link.sense_id, 1);
        assert_eq!(link.target_word_id, 20);
    }

    #[tokio::test]
    async fn create_sense_word_link_sense_not_found() {
        let service = AssocService::new(
            StubWordRepository::without_sense(),
            StubGraphRepository::success(),
        );
        let err = service
            .create_sense_word_link(1, 999, 20, SenseWordLinkKind::Synonym, None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::BusinessError(BusinessError::Link(LinkError::TargetNotFound))
        ));
    }

    #[tokio::test]
    async fn create_sense_word_link_target_word_not_found() {
        let service = AssocService::new(
            StubWordRepository::without_word(),
            StubGraphRepository::success(),
        );
        let err = service
            .create_sense_word_link(1, 1, 999, SenseWordLinkKind::Synonym, None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::BusinessError(BusinessError::Link(LinkError::TargetNotFound))
        ));
    }

    #[tokio::test]
    async fn create_sense_word_link_self_link_error() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        // source_word_id is 10 (from find_sense_by_id), target_word_id is also 10
        let err = service
            .create_sense_word_link(1, 1, 10, SenseWordLinkKind::Synonym, None)
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            AppError::BusinessError(BusinessError::Link(LinkError::SelfForbidden))
        ));
    }

    #[tokio::test]
    async fn list_word_links_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .list_word_links(WordLinkFilter {
                user_id: 1,
                word_id: 10,
                kind: None,
                limit: 20,
                offset: 0,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_sense_word_links_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .list_sense_word_links(SenseLinkFilter {
                user_id: 1,
                sense_id: 1,
                kind: None,
                limit: 20,
                offset: 0,
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_word_link_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .delete_word_link(1, 10, 20, WordLinkKind::SimilarForm)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_sense_word_link_success() {
        let service = AssocService::new(
            StubWordRepository::with_sense_and_word(),
            StubGraphRepository::success(),
        );
        let result = service
            .delete_sense_word_link(1, 1, 20, SenseWordLinkKind::Synonym)
            .await;
        assert!(result.is_ok());
    }
}

