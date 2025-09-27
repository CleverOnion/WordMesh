use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

pub const MAX_TAGS: usize = 20;
pub const MAX_NOTE_LENGTH: usize = 512;
pub const MAX_SENSE_TEXT_LENGTH: usize = 512;
pub const MAX_SENSE_NOTE_LENGTH: usize = 512;

static TAG_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_-]{1,24}$").unwrap());

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("value cannot be blank")]
    Blank,
    #[error("text too long: {0} characters (max {MAX_SENSE_TEXT_LENGTH})")]
    TextTooLong(usize),
    #[error("note too long: {0} characters (max {MAX_NOTE_LENGTH})")]
    NoteTooLong(usize),
    #[error("invalid tag: {0}")]
    InvalidTag(String),
    #[error("tag limit exceeded: {0} tags provided (max {MAX_TAGS})")]
    TagLimitExceeded(usize),
}

pub fn validate_non_empty_text(text: impl AsRef<str>) -> Result<String, ValidationError> {
    let value = text.as_ref().trim();
    if value.is_empty() {
        return Err(ValidationError::Blank);
    }
    if value.chars().count() > MAX_SENSE_TEXT_LENGTH {
        return Err(ValidationError::TextTooLong(value.chars().count()));
    }
    Ok(value.to_string())
}

pub fn validate_note(note: Option<String>) -> Result<Option<String>, ValidationError> {
    match note {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ValidationError::Blank);
            }
            let length = trimmed.chars().count();
            if length > MAX_NOTE_LENGTH {
                return Err(ValidationError::NoteTooLong(length));
            }
            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

pub fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>, ValidationError> {
    let mut seen = std::collections::HashSet::new();
    let mut normalized = Vec::new();
    for raw in tags {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::InvalidTag(raw));
        }
        if !TAG_REGEX.is_match(trimmed) {
            return Err(ValidationError::InvalidTag(trimmed.to_string()));
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            normalized.push(trimmed.to_string());
        }
    }
    if normalized.len() > MAX_TAGS {
        return Err(ValidationError::TagLimitExceeded(normalized.len()));
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_non_empty_text_ok() {
        assert_eq!(validate_non_empty_text(" hello ").unwrap(), "hello");
    }

    #[test]
    fn validate_non_empty_text_blank() {
        let err = validate_non_empty_text("   ").unwrap_err();
        assert_eq!(err, ValidationError::Blank);
    }

    #[test]
    fn validate_note_checks_length() {
        let err = validate_note(Some(" ".into())).unwrap_err();
        assert_eq!(err, ValidationError::Blank);
    }

    #[test]
    fn normalize_tags_enforces_rules() {
        let tags = vec!["tag-one".into(), "Tag-One".into(), "tag_two".into()];
        let normalized = normalize_tags(tags).unwrap();
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0], "tag-one");
        assert_eq!(normalized[1], "tag_two");
    }
}
