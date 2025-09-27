use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use neo4rs::{query, Graph, Node, Relation};
use thiserror::Error;
use tokio::time::{timeout, Duration};

use crate::config::settings::Neo4jSettings;
use crate::util::error::{BusinessError, LinkError};

pub type GraphResult<T> = Result<T, GraphRepositoryError>;

#[derive(Debug, Error)]
pub enum GraphRepositoryError {
    #[error("neo4j error: {0}")]
    Database(#[from] neo4rs::Error),
    #[error("timeout")]
    Timeout,
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("business error: {0}")]
    Business(#[from] BusinessError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordLinkKind {
    SimilarForm,
    RootAffix,
}

impl WordLinkKind {
    fn as_str(self) -> &'static str {
        match self {
            WordLinkKind::SimilarForm => "similar_form",
            WordLinkKind::RootAffix => "root_affix",
        }
    }

    fn try_from_str(value: &str) -> Option<Self> {
        match value {
            "similar_form" => Some(Self::SimilarForm),
            "root_affix" => Some(Self::RootAffix),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SenseWordLinkKind {
    Synonym,
    Antonym,
    Related,
}

impl SenseWordLinkKind {
    fn as_str(self) -> &'static str {
        match self {
            SenseWordLinkKind::Synonym => "synonym",
            SenseWordLinkKind::Antonym => "antonym",
            SenseWordLinkKind::Related => "related",
        }
    }

    fn try_from_str(value: &str) -> Option<Self> {
        match value {
            "synonym" => Some(Self::Synonym),
            "antonym" => Some(Self::Antonym),
            "related" => Some(Self::Related),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WordLinkRecord {
    pub link_id: String,
    pub user_id: i64,
    pub kind: WordLinkKind,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub word_a_id: i64,
    pub word_b_id: i64,
}

#[derive(Debug, Clone)]
pub struct SenseWordLinkRecord {
    pub link_id: String,
    pub user_id: i64,
    pub kind: SenseWordLinkKind,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sense_id: i64,
    pub source_word_id: i64,
    pub target_word_id: i64,
}

#[derive(Debug, Clone)]
pub struct WordLinkFilter {
    pub user_id: i64,
    pub kind: Option<WordLinkKind>,
    pub word_id: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct SenseLinkFilter {
    pub user_id: i64,
    pub sense_id: i64,
    pub kind: Option<SenseWordLinkKind>,
    pub limit: i64,
    pub offset: i64,
}

#[async_trait]
pub trait GraphRepository: Send + Sync {
    async fn create_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
        note: Option<String>,
    ) -> GraphResult<WordLinkRecord>;

    async fn delete_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
    ) -> GraphResult<()>;

    async fn list_word_links(&self, filter: WordLinkFilter) -> GraphResult<Vec<WordLinkRecord>>;

    async fn create_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        source_word_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
        note: Option<String>,
    ) -> GraphResult<SenseWordLinkRecord>;

    async fn delete_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
    ) -> GraphResult<()>;

    async fn list_sense_word_links(&self, filter: SenseLinkFilter) -> GraphResult<Vec<SenseWordLinkRecord>>;

    async fn remove_links_for_sense(&self, sense_id: i64) -> GraphResult<()>;

    async fn upsert_node_word(&self, word_id: i64) -> GraphResult<()>;

    async fn upsert_node_sense(&self, sense_id: i64, user_id: i64) -> GraphResult<()>;
}

#[derive(Clone)]
pub struct Neo4jGraphRepository {
    graph: Arc<Graph>,
    timeout: Duration,
}

impl Neo4jGraphRepository {
    pub async fn from_settings(settings: &Neo4jSettings) -> Result<Self, neo4rs::Error> {
        let graph = Graph::new(&settings.uri, settings.username.clone(), settings.password.clone())?;
        Ok(Self {
            graph: Arc::new(graph),
            timeout: Duration::from_secs(settings.query_timeout_seconds),
        })
    }

    pub fn new(graph: Graph, timeout: Duration) -> Self {
        Self {
            graph: Arc::new(graph),
            timeout,
        }
    }

    async fn run_with_timeout(&self, query: neo4rs::Query) -> GraphResult<Vec<neo4rs::Row>> {
        match timeout(self.timeout, self.graph.execute(query)).await {
            Ok(Ok(mut result)) => {
                let mut rows = Vec::new();
                while let Ok(Some(row)) = result.next().await {
                    rows.push(row);
                }
                Ok(rows)
            }
            Ok(Err(err)) => Err(GraphRepositoryError::Database(err)),
            Err(_) => Err(GraphRepositoryError::Timeout),
        }
    }

    fn parse_word_link(row: neo4rs::Row) -> GraphResult<WordLinkRecord> {
        let rel: Relation = row
            .get("rel")
            .map_err(|_| GraphRepositoryError::InvalidData("missing relationship".into()))?;
        let word_a: Node = row
            .get("word_a")
            .map_err(|_| GraphRepositoryError::InvalidData("missing word_a".into()))?;
        let word_b: Node = row
            .get("word_b")
            .map_err(|_| GraphRepositoryError::InvalidData("missing word_b".into()))?;
        let kind: String = rel
            .get("kind")
            .map_err(|_| GraphRepositoryError::InvalidData("missing kind field on relationship".into()))?;
        Ok(WordLinkRecord {
            link_id: rel.id().to_string(),
            user_id: rel
                .get("user_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing user_id on relationship".into()))?,
            kind: WordLinkKind::try_from_str(&kind)
                .ok_or_else(|| GraphRepositoryError::InvalidData("invalid word link kind".into()))?,
            note: rel.get("note").unwrap_or(None),
            created_at: Self::parse_datetime(&rel)?,
            word_a_id: word_a
                .get("word_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing word_id on word_a".into()))?,
            word_b_id: word_b
                .get("word_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing word_id on word_b".into()))?,
        })
    }

    fn parse_sense_word_link(row: neo4rs::Row) -> GraphResult<SenseWordLinkRecord> {
        let sense_node: Node = row
            .get("sense")
            .map_err(|_| GraphRepositoryError::InvalidData("missing sense node".into()))?;
        let word_node: Node = row
            .get("word")
            .map_err(|_| GraphRepositoryError::InvalidData("missing word node".into()))?;
        let rel: Relation = row
            .get("rel")
            .map_err(|_| GraphRepositoryError::InvalidData("missing relationship".into()))?;
        let kind_str: String = rel
            .get("kind")
            .map_err(|_| GraphRepositoryError::InvalidData("missing kind on relationship".into()))?;
        Ok(SenseWordLinkRecord {
            link_id: rel.id().to_string(),
            user_id: sense_node
                .get("user_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing user_id on sense".into()))?,
            kind: SenseWordLinkKind::try_from_str(&kind_str).ok_or_else(|| {
                GraphRepositoryError::InvalidData("invalid sense-word link kind".into())
            })?,
            note: rel.get("note").unwrap_or(None),
            created_at: Self::parse_datetime(&rel)?,
            sense_id: sense_node
                .get("sense_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing sense_id on sense".into()))?,
            source_word_id: sense_node
                .get("word_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing source word id on sense".into()))?,
            target_word_id: word_node
                .get("word_id")
                .map_err(|_| GraphRepositoryError::InvalidData("missing word_id on target word".into()))?,
        })
    }

    fn parse_datetime(rel: &Relation) -> GraphResult<DateTime<Utc>> {
        let dt = rel
            .get::<DateTime<Utc>>("created_at")
            .map_err(|_| GraphRepositoryError::InvalidData("missing created_at on relationship".into()))?;
        Ok(dt)
    }

    fn sort_word_ids(word_a_id: i64, word_b_id: i64) -> GraphResult<(i64, i64)> {
        if word_a_id == word_b_id {
            return Err(GraphRepositoryError::Business(BusinessError::from(
                LinkError::SelfForbidden,
            )));
        }
        if word_a_id < word_b_id {
            Ok((word_a_id, word_b_id))
        } else {
            Ok((word_b_id, word_a_id))
        }
    }
}

#[async_trait]
impl GraphRepository for Neo4jGraphRepository {
    async fn create_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
        note: Option<String>,
    ) -> GraphResult<WordLinkRecord> {
        let (min_id, max_id) = Self::sort_word_ids(word_a_id, word_b_id)?;
        let mut builder = query(
            "MERGE (a:Word { word_id: $min_id })\nMERGE (b:Word { word_id: $max_id })\nMERGE (a)-[r:WORD_TO_WORD { user_id: $user_id, kind: $kind }]->(b)\nON CREATE SET r.created_at = datetime(), r.note = $note\nON MATCH SET r.note = CASE WHEN $note IS NULL THEN r.note ELSE $note END\nRETURN a AS word_a, b AS word_b, r AS rel",
        )
        .param("min_id", min_id)
        .param("max_id", max_id)
        .param("user_id", user_id)
        .param("kind", kind.as_str())
        .param("note", note);

        let mut rows = self.run_with_timeout(builder).await?;
        if let Some(row) = rows.pop() {
            let mut record = Self::parse_word_link(row)?;
            record.word_a_id = word_a_id;
            record.word_b_id = word_b_id;
            Ok(record)
        } else {
            Err(GraphRepositoryError::InvalidData(
                "create_word_link returned no rows".into(),
            ))
        }
    }

    async fn delete_word_link(
        &self,
        user_id: i64,
        word_a_id: i64,
        word_b_id: i64,
        kind: WordLinkKind,
    ) -> GraphResult<()> {
        let (min_id, max_id) = Self::sort_word_ids(word_a_id, word_b_id)?;
        let query = query(
            "MATCH (a:Word { word_id: $min_id })-[r:WORD_TO_WORD { user_id: $user_id, kind: $kind }]->(b:Word { word_id: $max_id })\nDELETE r",
        )
        .param("min_id", min_id)
        .param("max_id", max_id)
        .param("user_id", user_id)
        .param("kind", kind.as_str());
        self.run_with_timeout(query).await.map(|_| ())
    }

    async fn list_word_links(&self, filter: WordLinkFilter) -> GraphResult<Vec<WordLinkRecord>> {
        let mut builder = query(
            "MATCH (word_a:Word { word_id: $word_id })-[rel:WORD_TO_WORD { user_id: $user_id }]->(word_b:Word)\nWHERE rel.kind IN $kinds\nRETURN word_a, word_b, rel\nORDER BY rel.created_at DESC\nSKIP $offset LIMIT $limit",
        )
        .param("word_id", filter.word_id)
        .param("user_id", filter.user_id)
        .param("offset", filter.offset)
        .param("limit", filter.limit);

        let kinds: Vec<&str> = match filter.kind {
            Some(kind) => vec![kind.as_str()],
            None => vec![WordLinkKind::SimilarForm.as_str(), WordLinkKind::RootAffix.as_str()],
        };

        builder = builder.param("kinds", kinds);

        let rows = self.run_with_timeout(builder).await?;
        rows.into_iter().map(Self::parse_word_link).collect()
    }

    async fn create_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        source_word_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
        note: Option<String>,
    ) -> GraphResult<SenseWordLinkRecord> {
        if source_word_id == target_word_id {
            return Err(GraphRepositoryError::Business(BusinessError::from(
                LinkError::SelfForbidden,
            )));
        }

        let mut builder = query(
            "MERGE (sense:UserSense { sense_id: $sense_id, user_id: $user_id })\nMERGE (target:Word { word_id: $target_word_id })\nMERGE (sense)-[rel:SENSE_TO_WORD { user_id: $user_id, kind: $kind }]->(target)\nON CREATE SET rel.created_at = datetime(), rel.note = $note\nON MATCH SET rel.note = CASE WHEN $note IS NULL THEN rel.note ELSE $note END\nRETURN sense, target AS word, rel",
        )
        .param("sense_id", sense_id)
        .param("user_id", user_id)
        .param("target_word_id", target_word_id)
        .param("kind", kind.as_str())
        .param("note", note);

        let rows = self.run_with_timeout(builder).await?;
        let mut record = rows
            .into_iter()
            .next()
            .map(Self::parse_sense_word_link)
            .transpose()?
            .ok_or_else(|| {
                GraphRepositoryError::InvalidData("create_sense_word_link returned no rows".into())
            })?;

        if record.source_word_id != source_word_id {
            record.source_word_id = source_word_id;
        }
        Ok(record)
    }

    async fn delete_sense_word_link(
        &self,
        user_id: i64,
        sense_id: i64,
        target_word_id: i64,
        kind: SenseWordLinkKind,
    ) -> GraphResult<()> {
        let query = query(
            "MATCH (sense:UserSense { sense_id: $sense_id, user_id: $user_id })-[rel:SENSE_TO_WORD { kind: $kind }]->(word:Word { word_id: $target_word_id })\nDELETE rel",
        )
        .param("sense_id", sense_id)
        .param("user_id", user_id)
        .param("target_word_id", target_word_id)
        .param("kind", kind.as_str());

        self.run_with_timeout(query).await.map(|_| ())
    }

    async fn list_sense_word_links(&self, filter: SenseLinkFilter) -> GraphResult<Vec<SenseWordLinkRecord>> {
        let mut builder = query(
            "MATCH (sense:UserSense { sense_id: $sense_id, user_id: $user_id })-[rel:SENSE_TO_WORD]->(word:Word)\nWHERE rel.kind IN $kinds\nRETURN sense, word, rel\nORDER BY rel.created_at DESC\nSKIP $offset LIMIT $limit",
        )
        .param("sense_id", filter.sense_id)
        .param("user_id", filter.user_id)
        .param("offset", filter.offset)
        .param("limit", filter.limit);

        let kinds: Vec<&str> = match filter.kind {
            Some(kind) => vec![kind.as_str()],
            None => vec![
                SenseWordLinkKind::Synonym.as_str(),
                SenseWordLinkKind::Antonym.as_str(),
                SenseWordLinkKind::Related.as_str(),
            ],
        };

        builder = builder.param("kinds", kinds);

        let rows = self.run_with_timeout(builder).await?;
        rows.into_iter().map(Self::parse_sense_word_link).collect()
    }

    async fn remove_links_for_sense(&self, sense_id: i64) -> GraphResult<()> {
        let query = query(
            "MATCH (:UserSense { sense_id: $sense_id })-[rel:SENSE_TO_WORD]->()\nDELETE rel",
        )
        .param("sense_id", sense_id);
        self.run_with_timeout(query).await.map(|_| ())
    }

    async fn upsert_node_word(&self, word_id: i64) -> GraphResult<()> {
        let query = query("MERGE (:Word { word_id: $word_id })")
            .param("word_id", word_id);
        self.run_with_timeout(query).await.map(|_| ())
    }

    async fn upsert_node_sense(&self, sense_id: i64, user_id: i64) -> GraphResult<()> {
        let query = query(
            "MERGE (:UserSense { sense_id: $sense_id, user_id: $user_id })",
        )
        .param("sense_id", sense_id)
        .param("user_id", user_id);
        self.run_with_timeout(query).await.map(|_| ())
    }
}
