use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::repository::graph::{SenseWordLinkKind, WordLinkKind};
use crate::repository::word::SearchScope;
use crate::service::sense::SenseUpdateInput;
use crate::service::word::{AddWordInput, SearchOptions, SenseInput};

#[derive(Debug, Deserialize, Validate)]
pub struct AddWordRequest {
    #[validate(length(min = 1, max = 128))]
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<String>,
    #[serde(default)]
    pub first_sense: Option<SenseRequest>,
}

impl From<AddWordRequest> for AddWordInput {
    fn from(value: AddWordRequest) -> Self {
        Self {
            text: value.text,
            tags: value.tags,
            note: value.note,
            first_sense: value.first_sense.map(Into::into),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SenseRequest {
    #[validate(length(min = 1, max = 512))]
    pub text: String,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<String>,
}

impl From<SenseRequest> for SenseInput {
    fn from(value: SenseRequest) -> Self {
        Self {
            text: value.text,
            is_primary: value.is_primary,
            sort_order: value.sort_order,
            note: value.note,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SearchRequest {
    #[serde(default)]
    #[validate(length(max = 128))]
    pub q: String,
    #[serde(default)]
    pub scope: Option<SearchScopeDto>,
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: i64,
    #[serde(default)]
    #[validate(range(min = 0, max = 10_000))]
    pub offset: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SearchScopeDto {
    Word,
    Sense,
    Both,
}

impl From<SearchScopeDto> for SearchScope {
    fn from(value: SearchScopeDto) -> Self {
        match value {
            SearchScopeDto::Word => SearchScope::Word,
            SearchScopeDto::Sense => SearchScope::Sense,
            SearchScopeDto::Both => SearchScope::Both,
        }
    }
}

impl SearchRequest {
    pub fn into_options(self) -> SearchOptions {
        SearchOptions {
            query: self.q,
            scope: self.scope.unwrap_or(SearchScopeDto::Both).into(),
            limit: self.limit,
            offset: self.offset,
        }
    }
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddSenseRequest {
    #[validate(length(min = 1, max = 512))]
    pub text: String,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<String>,
}

impl From<AddSenseRequest> for SenseInput {
    fn from(value: AddSenseRequest) -> Self {
        SenseInput {
            text: value.text,
            is_primary: value.is_primary,
            sort_order: value.sort_order,
            note: value.note,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSenseRequest {
    #[validate(length(min = 1, max = 512))]
    pub text: Option<String>,
    pub is_primary: Option<bool>,
    pub sort_order: Option<i32>,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<Option<String>>,
}

impl From<UpdateSenseRequest> for SenseUpdateInput {
    fn from(value: UpdateSenseRequest) -> Self {
        SenseUpdateInput {
            text: value.text,
            is_primary: value.is_primary,
            sort_order: value.sort_order,
            note: value.note,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWordLinkRequest {
    pub word_a_id: i64,
    pub word_b_id: i64,
    pub kind: WordLinkKindDto,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum WordLinkKindDto {
    SimilarForm,
    RootAffix,
}

impl From<WordLinkKindDto> for WordLinkKind {
    fn from(value: WordLinkKindDto) -> Self {
        match value {
            WordLinkKindDto::SimilarForm => WordLinkKind::SimilarForm,
            WordLinkKindDto::RootAffix => WordLinkKind::RootAffix,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSenseWordLinkRequest {
    pub sense_id: i64,
    pub target_word_id: i64,
    pub kind: SenseLinkKindDto,
    #[serde(default)]
    #[validate(length(min = 1, max = 512))]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum SenseLinkKindDto {
    Synonym,
    Antonym,
    Related,
}

impl From<SenseLinkKindDto> for SenseWordLinkKind {
    fn from(value: SenseLinkKindDto) -> Self {
        match value {
            SenseLinkKindDto::Synonym => SenseWordLinkKind::Synonym,
            SenseLinkKindDto::Antonym => SenseWordLinkKind::Antonym,
            SenseLinkKindDto::Related => SenseWordLinkKind::Related,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LinkQuery {
    pub endpoint_type: LinkEndpointDto,
    pub endpoint_id: i64,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: i64,
    #[serde(default)]
    #[validate(range(min = 0, max = 10_000))]
    pub offset: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LinkEndpointDto {
    Word,
    Sense,
}
