use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use thiserror::Error;

const MAX_TAGS: usize = 20;
const MAX_NOTE_LENGTH: usize = 512;
const MAX_SENSE_TEXT_LENGTH: usize = 512;
const MAX_SENSE_NOTE_LENGTH: usize = 512;

static MULTI_WHITESPACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\s+").expect("canonical key whitespace regex must compile"));
static TAG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_-]{1,24}$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalKey(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanonicalKeyError {
    #[error("canonical key text cannot be empty after normalization")]
    Empty,
}

impl CanonicalKey {
    pub fn new(text: impl AsRef<str>) -> Result<Self, CanonicalKeyError> {
        let normalized = normalize_canonical_text(text.as_ref());
        if normalized.is_empty() {
            return Err(CanonicalKeyError::Empty);
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CanonicalKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn normalize_canonical_text(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let collapsed = MULTI_WHITESPACE.replace_all(trimmed, " ");
    let stripped = collapsed
        .trim_matches(|c: char| c.is_ascii_punctuation())
        .trim();
    if stripped.is_empty() {
        return String::new();
    }

    let lowercase = stripped.to_lowercase();
    let replaced = lowercase.replace(' ', "-");
    let mut cleaned = String::with_capacity(replaced.len());
    let mut last_dash = false;
    for ch in replaced.chars() {
        if ch == '-' {
            if !last_dash {
                cleaned.push('-');
                last_dash = true;
            }
        } else if ch.is_ascii_punctuation() {
            continue;
        } else {
            cleaned.push(ch);
            last_dash = false;
        }
    }
    cleaned.trim_matches('-').to_string()
}

#[derive(Debug, Clone)]
pub struct UserWord {
    pub id: Option<i64>,
    pub user_id: i64,
    pub word_id: i64,
    tags: Vec<String>,
    note: Option<String>,
    senses: Vec<UserSense>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserSense {
    pub id: Option<i64>,
    text: String,
    pub is_primary: bool,
    pub sort_order: i32,
    note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum UserWordError {
    #[error("tag limit exceeded: {0} tags provided (max {MAX_TAGS})")]
    TagLimitExceeded(usize),
    #[error("invalid tag: {0}")]
    InvalidTag(String),
    #[error("note cannot be blank")]
    InvalidNote,
    #[error("note too long: {0} characters (max {MAX_NOTE_LENGTH})")]
    NoteTooLong(usize),
    #[error("duplicate sense text detected: {0}")]
    DuplicateSenseText(String),
    #[error("sense with id {0} not found")]
    SenseNotFound(i64),
    #[error("sense index {0} out of bounds")]
    SenseIndexOutOfBounds(usize),
    #[error(transparent)]
    Sense(#[from] UserSenseError),
}

#[derive(Debug, Error)]
pub enum UserSenseError {
    #[error("sense text cannot be empty")]
    EmptyText,
    #[error("sense text too long: {0} characters (max {MAX_SENSE_TEXT_LENGTH})")]
    TextTooLong(usize),
    #[error("sense note cannot be blank")]
    InvalidNote,
    #[error("sense note too long: {0} characters (max {MAX_SENSE_NOTE_LENGTH})")]
    NoteTooLong(usize),
}

impl UserWord {
    pub fn create(
        user_id: i64,
        word_id: i64,
        tags: Vec<String>,
        note: Option<String>,
    ) -> Result<Self, UserWordError> {
        let tags = normalize_tags(tags)?;
        let note = validate_note(note)?;
        Ok(Self {
            id: None,
            user_id,
            word_id,
            tags,
            note,
            senses: Vec::new(),
            created_at: Utc::now(),
        })
    }

    pub fn from_parts(
        id: Option<i64>,
        user_id: i64,
        word_id: i64,
        tags: Vec<String>,
        note: Option<String>,
        senses: Vec<UserSense>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, UserWordError> {
        let tags = normalize_tags(tags)?;
        let note = validate_note(note)?;
        let mut word = Self {
            id,
            user_id,
            word_id,
            tags,
            note,
            senses: Vec::new(),
            created_at,
        };
        for sense in senses {
            word.add_sense(sense)?;
        }
        Ok(word)
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn senses(&self) -> &[UserSense] {
        &self.senses
    }

    pub fn senses_mut_for_test(&mut self) -> &mut Vec<UserSense> {
        &mut self.senses
    }

    pub fn update_tags(&mut self, tags: Vec<String>) -> Result<(), UserWordError> {
        self.tags = normalize_tags(tags)?;
        Ok(())
    }

    pub fn update_note(&mut self, note: Option<String>) -> Result<(), UserWordError> {
        self.note = validate_note(note)?;
        Ok(())
    }

    pub fn add_sense(&mut self, mut sense: UserSense) -> Result<(), UserWordError> {
        if self
            .senses
            .iter()
            .any(|existing| existing.text.eq_ignore_ascii_case(&sense.text))
        {
            return Err(UserWordError::DuplicateSenseText(sense.text.clone()));
        }

        if sense.is_primary {
            self.clear_primary();
        }

        if sense.sort_order == i32::MIN {
            sense.sort_order = 0;
        }

        self.senses.push(sense);
        self.senses.sort_by_key(|s| s.sort_order);
        Ok(())
    }

    pub fn set_primary_by_id(&mut self, sense_id: i64) -> Result<(), UserWordError> {
        let mut found = false;
        for sense in &mut self.senses {
            if sense.id == Some(sense_id) {
                found = true;
                sense.is_primary = true;
            } else {
                sense.is_primary = false;
            }
        }
        if !found {
            return Err(UserWordError::SenseNotFound(sense_id));
        }
        Ok(())
    }

    pub fn set_primary_by_index(&mut self, index: usize) -> Result<(), UserWordError> {
        if index >= self.senses.len() {
            return Err(UserWordError::SenseIndexOutOfBounds(index));
        }
        for (idx, sense) in self.senses.iter_mut().enumerate() {
            sense.is_primary = idx == index;
        }
        Ok(())
    }

    pub fn remove_sense_by_id(&mut self, sense_id: i64) -> Result<UserSense, UserWordError> {
        if let Some(pos) = self
            .senses
            .iter()
            .position(|sense| sense.id == Some(sense_id))
        {
            Ok(self.senses.remove(pos))
        } else {
            Err(UserWordError::SenseNotFound(sense_id))
        }
    }

    pub fn clear_primary(&mut self) {
        for sense in &mut self.senses {
            sense.is_primary = false;
        }
    }
}

impl UserSense {
    pub fn new(
        text: impl Into<String>,
        is_primary: bool,
        sort_order: i32,
        note: Option<String>,
    ) -> Result<Self, UserSenseError> {
        let text = normalize_sense_text(text.into())?;
        let note = validate_sense_note(note)?;
        Ok(Self {
            id: None,
            text,
            is_primary,
            sort_order,
            note,
            created_at: Utc::now(),
        })
    }

    pub fn from_parts(
        id: Option<i64>,
        text: String,
        is_primary: bool,
        sort_order: i32,
        note: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, UserSenseError> {
        let text = normalize_sense_text(text)?;
        let note = validate_sense_note(note)?;
        Ok(Self {
            id,
            text,
            is_primary,
            sort_order,
            note,
            created_at,
        })
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn set_text(&mut self, text: impl Into<String>) -> Result<(), UserSenseError> {
        self.text = normalize_sense_text(text.into())?;
        Ok(())
    }

    pub fn set_note(&mut self, note: Option<String>) -> Result<(), UserSenseError> {
        self.note = validate_sense_note(note)?;
        Ok(())
    }

    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
    }

    pub fn set_primary(&mut self, is_primary: bool) {
        self.is_primary = is_primary;
    }
}

fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>, UserWordError> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for raw in tags {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(UserWordError::InvalidTag(raw));
        }
        if !TAG_REGEX.is_match(trimmed) {
            return Err(UserWordError::InvalidTag(trimmed.to_string()));
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            normalized.push(trimmed.to_string());
        }
    }
    if normalized.len() > MAX_TAGS {
        return Err(UserWordError::TagLimitExceeded(normalized.len()));
    }
    Ok(normalized)
}

fn validate_note(note: Option<String>) -> Result<Option<String>, UserWordError> {
    match note {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(UserWordError::InvalidNote);
            }
            if trimmed.chars().count() > MAX_NOTE_LENGTH {
                return Err(UserWordError::NoteTooLong(trimmed.chars().count()));
            }
            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

fn normalize_sense_text(text: String) -> Result<String, UserSenseError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(UserSenseError::EmptyText);
    }
    let length = trimmed.chars().count();
    if length > MAX_SENSE_TEXT_LENGTH {
        return Err(UserSenseError::TextTooLong(length));
    }
    Ok(trimmed.to_string())
}

fn validate_sense_note(note: Option<String>) -> Result<Option<String>, UserSenseError> {
    match note {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(UserSenseError::InvalidNote);
            }
            let length = trimmed.chars().count();
            if length > MAX_SENSE_NOTE_LENGTH {
                return Err(UserSenseError::NoteTooLong(length));
            }
            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_key_normalizes_text() {
        let key = CanonicalKey::new("  Graph   Database  ").unwrap();
        assert_eq!(key.as_str(), "graph-database");
    }

    #[test]
    fn canonical_key_rejects_empty_input() {
        let result = CanonicalKey::new("   ");
        assert!(matches!(result, Err(CanonicalKeyError::Empty)));
    }

    #[test]
    fn canonical_key_strips_punctuation() {
        let key = CanonicalKey::new("**Hello, World!!").unwrap();
        assert_eq!(key.as_str(), "hello-world");
    }

    #[test]
    fn user_word_creates_with_valid_tags() {
        let word = UserWord::create(
            1,
            10,
            vec!["tag-one".into(), "Tag-One".into(), "tag_two".into()],
            Some(" personal note ".into()),
        )
        .unwrap();
        assert_eq!(word.tags.len(), 2);
        assert_eq!(word.tags[0], "tag-one");
        assert_eq!(word.tags[1], "tag_two");
        assert_eq!(word.note(), Some("personal note"));
    }

    #[test]
    fn user_word_rejects_too_many_tags() {
        let tags = (0..25).map(|i| format!("tag{i}")).collect::<Vec<_>>();
        let result = UserWord::create(1, 1, tags, None);
        assert!(matches!(result, Err(UserWordError::TagLimitExceeded(_))));
    }

    #[test]
    fn user_word_rejects_invalid_tag() {
        let result = UserWord::create(1, 1, vec!["bad tag".into()], None);
        assert!(matches!(result, Err(UserWordError::InvalidTag(_))));
    }

    #[test]
    fn user_sense_creation_and_addition_preserves_primary_uniqueness() {
        let mut word = UserWord::create(1, 1, vec![], None).unwrap();
        let primary = UserSense::new("meaning", true, 0, None).unwrap();
        let secondary = UserSense::new("second", false, 1, None).unwrap();

        word.add_sense(primary).unwrap();
        word.add_sense(secondary).unwrap();

        assert_eq!(word.senses().len(), 2);
        assert!(word.senses()[0].is_primary);
        assert!(!word.senses()[1].is_primary);
    }

    #[test]
    fn user_word_set_primary_by_index_updates_all_senses() {
        let mut word = UserWord::create(1, 1, vec![], None).unwrap();
        let first = UserSense::new("first", true, 0, None).unwrap();
        let mut second = UserSense::new("second", false, 1, None).unwrap();
        second.id = Some(2);

        word.add_sense(first).unwrap();
        word.add_sense(second).unwrap();

        word.set_primary_by_index(1).unwrap();
        assert!(!word.senses()[0].is_primary);
        assert!(word.senses()[1].is_primary);

        word.set_primary_by_id(2).unwrap();
        assert!(!word.senses()[0].is_primary);
        assert!(word.senses()[1].is_primary);
    }

    #[test]
    fn user_word_detects_duplicate_sense_text() {
        let mut word = UserWord::create(1, 1, vec![], None).unwrap();
        word.add_sense(UserSense::new("duplicate", false, 0, None).unwrap())
            .unwrap();
        let result = word.add_sense(UserSense::new("duplicate", false, 1, None).unwrap());
        assert!(matches!(result, Err(UserWordError::DuplicateSenseText(_))));
    }

    #[test]
    fn user_sense_validates_note_and_text() {
        let result = UserSense::new("   ", false, 0, None);
        assert!(matches!(result, Err(UserSenseError::EmptyText)));

        let result = UserSense::new("meaning", false, 0, Some("   ".into()));
        assert!(matches!(result, Err(UserSenseError::InvalidNote)));
    }
}
