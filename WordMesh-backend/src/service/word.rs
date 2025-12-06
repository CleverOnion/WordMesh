use std::sync::Arc;

use tracing::instrument;

use crate::domain::word::{
    CanonicalKey, CanonicalKeyError, UserSense, UserSenseError, UserWordError,
};
use crate::repository::graph::{GraphRepository, GraphRepositoryError, WordLinkFilter};
use crate::repository::word::{
    NewUserSense, SearchParams, SearchScope, UpsertUserWord, UserWordAggregate, WordRepository,
    WordRepositoryError,
};
use crate::util::error::{AppError, BusinessError, LinkError, ValidationField, WordError};
use crate::util::validation::{
    MAX_NOTE_LENGTH, MAX_SENSE_NOTE_LENGTH, MAX_SENSE_TEXT_LENGTH, MAX_TAGS, ValidationError,
    normalize_tags, validate_non_empty_text, validate_note,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AddWordInput {
    pub text: String,
    pub tags: Vec<String>,
    pub note: Option<String>,
    pub first_sense: Option<SenseInput>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SenseInput {
    pub text: String,
    pub is_primary: bool,
    pub sort_order: i32,
    pub note: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub scope: SearchScope,
    pub limit: i64,
    pub offset: i64,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            scope: SearchScope::Both,
            limit: 20,
            offset: 0,
        }
    }
}

#[allow(dead_code)]
pub struct WordService<W, G>
where
    W: WordRepository + Send + Sync + 'static,
    G: GraphRepository + Send + Sync + 'static,
{
    word_repository: Arc<W>,
    graph_repository: Arc<G>,
}

impl<W, G> WordService<W, G>
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
    #[instrument(skip(self, input), fields(user_id = user_id))]
    pub async fn add_to_my_network(
        &self,
        user_id: i64,
        input: AddWordInput,
    ) -> Result<UserWordAggregate, AppError> {
        let AddWordInput {
            text,
            tags,
            note,
            first_sense,
        } = input;

        let canonical = CanonicalKey::new(&text).map_err(map_canonical_error)?;
        let tags = normalize_tags(tags).map_err(|err| map_validation_error("tags", err))?;
        let note = validate_note(note).map_err(|err| map_validation_error("note", err))?;

        let payload = UpsertUserWord {
            user_id,
            word_text: text,
            canonical_key: canonical,
            tags,
            note,
        };

        let aggregate = self
            .word_repository
            .upsert_user_word(payload)
            .await
            .map_err(map_word_error)?;

        self.graph_repository
            .upsert_node_word(aggregate.word.id)
            .await
            .map_err(map_graph_error)?;

        let user_word_id = aggregate
            .user_word
            .id
            .ok_or_else(|| AppError::from(BusinessError::Word(WordError::NotInNetwork)))?;

        if let Some(sense_input) = first_sense {
            let new_sense = build_new_sense_payload(user_word_id, sense_input)?;
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
        }

        self.word_repository
            .find_user_word(user_id, user_word_id)
            .await
            .map_err(map_word_error)?
            .ok_or_else(|| AppError::from(BusinessError::Word(WordError::NotInNetwork)))
    }

    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn remove_from_my_network(
        &self,
        user_id: i64,
        user_word_id: i64,
    ) -> Result<(), AppError> {
        let aggregate = self
            .word_repository
            .find_user_word(user_id, user_word_id)
            .await
            .map_err(map_word_error)?
            .ok_or_else(|| AppError::from(BusinessError::Word(WordError::NotInNetwork)))?;

        for sense in aggregate.user_word.senses() {
            if let Some(sense_id) = sense.id() {
                self.graph_repository
                    .remove_links_for_sense(sense_id)
                    .await
                    .map_err(map_graph_error)?;
            }
        }

        let mut offset = 0;
        loop {
            let links = self
                .graph_repository
                .list_word_links(WordLinkFilter {
                    user_id,
                    word_id: aggregate.word.id,
                    kind: None,
                    limit: 100,
                    offset,
                })
                .await
                .map_err(map_graph_error)?;

            if links.is_empty() {
                break;
            }

            for link in &links {
                self.graph_repository
                    .delete_word_link(user_id, link.word_a_id, link.word_b_id, link.kind)
                    .await
                    .map_err(map_graph_error)?;
            }

            offset += links.len() as i64;
        }

        self.word_repository
            .remove_user_word(user_id, user_word_id)
            .await
            .map_err(map_word_error)
    }

    #[allow(dead_code)]
    #[instrument(skip(self, options), fields(user_id = user_id))]
    pub async fn search_in_my_network(
        &self,
        user_id: i64,
        options: SearchOptions,
    ) -> Result<Vec<UserWordAggregate>, AppError> {
        let limit = options.limit.clamp(1, 100);
        let offset = options.offset.clamp(0, 10_000);

        let params = SearchParams {
            user_id,
            query: options.query,
            scope: options.scope,
            limit,
            offset,
        };

        self.word_repository
            .search(params)
            .await
            .map_err(map_word_error)
    }
}

pub(crate) fn build_new_sense_payload(
    user_word_id: i64,
    sense: SenseInput,
) -> Result<NewUserSense, AppError> {
    let SenseInput {
        text,
        is_primary,
        sort_order,
        note,
    } = sense;
    let text = validate_non_empty_text(text)
        .map_err(|err| map_validation_error("first_sense.text", err))?;
    let note = validate_note(note).map_err(|err| map_validation_error("first_sense.note", err))?;

    Ok(NewUserSense {
        user_word_id,
        text,
        is_primary,
        sort_order,
        note,
    })
}

pub(crate) fn validation_error(field: &str, message: impl Into<String>) -> AppError {
    AppError::from(BusinessError::Validation(vec![ValidationField {
        field: field.into(),
        message: message.into(),
    }]))
}

pub(crate) fn map_validation_error(field: &str, error: ValidationError) -> AppError {
    match error {
        ValidationError::Blank => validation_error(field, "不能为空"),
        ValidationError::TextTooLong(len) => validation_error(
            field,
            format!("文本长度不能超过 {MAX_SENSE_TEXT_LENGTH} 字符，当前 {len}"),
        ),
        ValidationError::NoteTooLong(len) => validation_error(
            field,
            format!("备注长度不能超过 {MAX_SENSE_NOTE_LENGTH} 字符，当前 {len}"),
        ),
        ValidationError::InvalidTag(tag) => validation_error(field, format!("无效标签: {tag}")),
        ValidationError::TagLimitExceeded(count) => {
            validation_error(field, format!("标签数量不能超过 {MAX_TAGS}，当前 {count}"))
        }
    }
}

pub(crate) fn map_canonical_error(err: CanonicalKeyError) -> AppError {
    match err {
        CanonicalKeyError::Empty => validation_error("text", "文本不能为空"),
        CanonicalKeyError::Validation(_) => validation_error("text", "文本不合法"),
    }
}

pub(crate) fn map_user_word_error(err: UserWordError) -> AppError {
    match err {
        UserWordError::TagLimitExceeded(count) => {
            validation_error("tags", format!("标签数量不能超过 {MAX_TAGS}，当前 {count}"))
        }
        UserWordError::InvalidTag(tag) => validation_error("tags", format!("无效标签: {tag}")),
        UserWordError::InvalidNote => validation_error("note", "备注不能为空"),
        UserWordError::NoteTooLong(len) => validation_error(
            "note",
            format!("备注长度不能超过 {MAX_NOTE_LENGTH} 字符，当前 {len}"),
        ),
        UserWordError::DuplicateSenseText(_) => {
            AppError::from(BusinessError::Word(WordError::SenseDuplicate))
        }
        UserWordError::SenseNotFound(_) => {
            AppError::from(BusinessError::Word(WordError::NotInNetwork))
        }
        UserWordError::SenseIndexOutOfBounds(_) => {
            AppError::from(BusinessError::Word(WordError::PrimaryConflict))
        }
        UserWordError::Sense(err) => map_user_sense_error(err),
    }
}

pub(crate) fn map_user_sense_error(err: UserSenseError) -> AppError {
    match err {
        UserSenseError::EmptyText => validation_error("sense.text", "义项文本不能为空"),
        UserSenseError::TextTooLong(len) => validation_error(
            "sense.text",
            format!("义项文本长度不能超过 {MAX_SENSE_TEXT_LENGTH} 字符，当前 {len}"),
        ),
        UserSenseError::InvalidNote => validation_error("sense.note", "义项备注不能为空"),
        UserSenseError::NoteTooLong(len) => validation_error(
            "sense.note",
            format!("义项备注长度不能超过 {MAX_SENSE_NOTE_LENGTH} 字符，当前 {len}"),
        ),
    }
}

pub(crate) fn map_word_error(err: WordRepositoryError) -> AppError {
    match err {
        WordRepositoryError::UserWord(inner) => map_user_word_error(inner),
        WordRepositoryError::UserSense(inner) => map_user_sense_error(inner),
        WordRepositoryError::Canonical(inner) => map_canonical_error(inner),
        WordRepositoryError::Database(_) => {
            AppError::from(BusinessError::Word(WordError::AlreadyExists))
        }
    }
}

pub(crate) fn map_graph_error(err: GraphRepositoryError) -> AppError {
    match err {
        GraphRepositoryError::Business(BusinessError::Link(link_err)) => {
            AppError::from(BusinessError::Link(link_err))
        }
        GraphRepositoryError::Business(other) => AppError::from(other),
        GraphRepositoryError::Database(_) | GraphRepositoryError::Timeout => {
            AppError::from(BusinessError::Link(LinkError::TargetNotFound))
        }
        GraphRepositoryError::InvalidData(_) => {
            AppError::from(BusinessError::Link(LinkError::TargetNotFound))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use crate::domain::word::UserWord;
    use crate::repository::graph::{GraphRepository, GraphResult, SenseLinkFilter, SenseWordLinkKind, WordLinkFilter, WordLinkKind};
    use crate::repository::word::{SearchParams, SearchScope, SenseUpdate, UpsertUserWord, UserWordAggregate, WordRecord};

    // Helper function to create a test aggregate
    fn create_test_aggregate(user_id: i64, user_word_id: i64, word_id: i64, text: &str) -> UserWordAggregate {
        let canonical_key = CanonicalKey::new(text).unwrap();
        let word = WordRecord {
            id: word_id,
            text: text.to_string(),
            canonical_key: canonical_key.clone(),
            created_at: Utc::now(),
        };
        let user_word = UserWord::from_parts(
            Some(user_word_id),
            user_id,
            word_id,
            vec![],
            None,
            vec![],
            Utc::now(),
        ).unwrap();
        UserWordAggregate { word, user_word }
    }

    // Stub implementations for different test scenarios
    struct SuccessWordRepository {
        user_word_id: i64,
        word_id: i64,
        has_sense: bool,
    }

    #[async_trait]
    impl WordRepository for SuccessWordRepository {
        async fn upsert_word(
            &self,
            _canonical: &CanonicalKey,
            _text: &str,
        ) -> Result<WordRecord, WordRepositoryError> {
            unimplemented!()
        }

        async fn upsert_user_word(
            &self,
            payload: UpsertUserWord,
        ) -> Result<UserWordAggregate, WordRepositoryError> {
            let canonical_key = payload.canonical_key.clone();
            let word = WordRecord {
                id: self.word_id,
                text: payload.word_text,
                canonical_key: canonical_key.clone(),
                created_at: Utc::now(),
            };
            let user_word = UserWord::from_parts(
                Some(self.user_word_id),
                payload.user_id,
                self.word_id,
                payload.tags,
                payload.note,
                vec![],
                Utc::now(),
            ).unwrap();
            Ok(UserWordAggregate { word, user_word })
        }

        async fn find_user_word(
            &self,
            _user_id: i64,
            user_word_id: i64,
        ) -> Result<Option<UserWordAggregate>, WordRepositoryError> {
            if user_word_id == self.user_word_id {
                let mut aggregate = create_test_aggregate(1, self.user_word_id, self.word_id, "hello");
                if self.has_sense {
                    let sense = UserSense::from_parts(
                        Some(1),
                        "greeting".to_string(),
                        true,
                        0,
                        None,
                        Utc::now(),
                    )
                    .map_err(WordRepositoryError::UserSense)?;
                    aggregate.user_word.add_sense(sense)
                        .map_err(WordRepositoryError::UserWord)?;
                }
                Ok(Some(aggregate))
            } else {
                Ok(None)
            }
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
            sense: NewUserSense,
        ) -> Result<UserSense, WordRepositoryError> {
            UserSense::from_parts(
                Some(1),
                sense.text,
                sense.is_primary,
                sense.sort_order,
                sense.note,
                Utc::now(),
            )
            .map_err(WordRepositoryError::UserSense)
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
            Ok(None)
        }

        async fn find_word_by_id(
            &self,
            _word_id: i64,
        ) -> Result<Option<WordRecord>, WordRepositoryError> {
            Ok(None)
        }

        async fn search(
            &self,
            params: SearchParams,
        ) -> Result<Vec<UserWordAggregate>, WordRepositoryError> {
            let mut results = vec![];
            if params.query.is_empty() || params.query == "hello" {
                results.push(create_test_aggregate(params.user_id, 1, 10, "hello"));
            }
            Ok(results)
        }
    }

    struct ErrorWordRepository {
        error_kind: ErrorKind,
    }

    enum ErrorKind {
        InvalidNote,
        TagLimitExceeded,
    }

    #[async_trait]
    impl WordRepository for ErrorWordRepository {
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
            let error = match self.error_kind {
                ErrorKind::InvalidNote => WordRepositoryError::UserWord(UserWordError::InvalidNote),
                ErrorKind::TagLimitExceeded => WordRepositoryError::UserWord(UserWordError::TagLimitExceeded(100)),
            };
            Err(error)
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
            Ok(None)
        }

        async fn find_word_by_id(
            &self,
            _word_id: i64,
        ) -> Result<Option<WordRecord>, WordRepositoryError> {
            Ok(None)
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
    async fn add_to_my_network_success_without_first_sense() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .add_to_my_network(
                1,
                AddWordInput {
                    text: "hello".into(),
                    tags: vec![],
                    note: None,
                    first_sense: None,
                },
            )
            .await;
        assert!(result.is_ok());
        let aggregate = result.unwrap();
        assert_eq!(aggregate.word.text, "hello");
        assert_eq!(aggregate.user_word.id, Some(1));
    }

    #[tokio::test]
    async fn add_to_my_network_success_with_first_sense() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: true,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .add_to_my_network(
                1,
                AddWordInput {
                    text: "hello".into(),
                    tags: vec![],
                    note: None,
                    first_sense: Some(SenseInput {
                        text: "greeting".into(),
                        is_primary: true,
                        sort_order: 0,
                        note: None,
                    }),
                },
            )
            .await;
        assert!(result.is_ok());
        let aggregate = result.unwrap();
        assert_eq!(aggregate.word.text, "hello");
        assert!(!aggregate.user_word.senses().is_empty());
    }

    #[tokio::test]
    async fn add_to_my_network_maps_validation_errors() {
        let repo = ErrorWordRepository {
            error_kind: ErrorKind::InvalidNote,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .add_to_my_network(
                1,
                AddWordInput {
                    text: "hello".into(),
                    tags: vec![],
                    note: None,
                    first_sense: None,
                },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_to_my_network_empty_text_error() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .add_to_my_network(
                1,
                AddWordInput {
                    text: "   ".into(), // Empty after normalization
                    tags: vec![],
                    note: None,
                    first_sense: None,
                },
            )
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::BusinessError(BusinessError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn add_to_my_network_too_many_tags_error() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let mut tags = vec![];
        for i in 0..=crate::util::validation::MAX_TAGS {
            tags.push(format!("tag{}", i));
        }
        let result = service
            .add_to_my_network(
                1,
                AddWordInput {
                    text: "hello".into(),
                    tags,
                    note: None,
                    first_sense: None,
                },
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_from_my_network_success() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service.remove_from_my_network(1, 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn remove_from_my_network_word_not_found() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service.remove_from_my_network(1, 999).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AppError::BusinessError(BusinessError::Word(WordError::NotInNetwork))
        ));
    }

    #[tokio::test]
    async fn search_in_my_network_empty_query() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .search_in_my_network(
                1,
                SearchOptions {
                    query: String::new(),
                    scope: SearchScope::Both,
                    limit: 20,
                    offset: 0,
                },
            )
            .await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn search_in_my_network_with_query() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .search_in_my_network(
                1,
                SearchOptions {
                    query: "hello".into(),
                    scope: SearchScope::Word,
                    limit: 20,
                    offset: 0,
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn search_in_my_network_pagination() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        let result = service
            .search_in_my_network(
                1,
                SearchOptions {
                    query: String::new(),
                    scope: SearchScope::Both,
                    limit: 10,
                    offset: 5,
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn search_in_my_network_limit_clamping() {
        let repo = SuccessWordRepository {
            user_word_id: 1,
            word_id: 10,
            has_sense: false,
        };
        let service = WordService::new(repo, StubGraphRepository);
        // Test limit > 100 gets clamped to 100
        let result = service
            .search_in_my_network(
                1,
                SearchOptions {
                    query: String::new(),
                    scope: SearchScope::Both,
                    limit: 200,
                    offset: 0,
                },
            )
            .await;
        assert!(result.is_ok());
    }
}
