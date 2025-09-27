use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

static MULTI_WHITESPACE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+").expect("canonical key whitespace regex must compile"));

/// Errors that can occur during canonical text normalization.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CanonicalError {
    #[error("canonical text cannot be empty after normalization")]
    Empty,
}

/// Convert arbitrary text into a canonical key format.
///
/// Normalization steps:
/// - trim leading/trailing whitespace
/// - collapse consecutive whitespace into a single space
/// - trim leading/trailing ASCII punctuation
/// - lowercase
/// - replace internal spaces with single hyphen (`-`)
/// - remove remaining ASCII punctuation, collapsing repeated hyphens
pub fn canonicalize(input: impl AsRef<str>) -> Result<String, CanonicalError> {
    let trimmed = input.as_ref().trim();
    if trimmed.is_empty() {
        return Err(CanonicalError::Empty);
    }

    let collapsed = MULTI_WHITESPACE.replace_all(trimmed, " ");
    let stripped = collapsed
        .trim_matches(|c: char| c.is_ascii_punctuation())
        .trim();
    if stripped.is_empty() {
        return Err(CanonicalError::Empty);
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

    let normalized = cleaned.trim_matches('-').to_string();
    if normalized.is_empty() {
        Err(CanonicalError::Empty)
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_basic() {
        let key = canonicalize("  Graph   Database  ").unwrap();
        assert_eq!(key, "graph-database");
    }

    #[test]
    fn canonicalize_rejects_empty() {
        let result = canonicalize("   ");
        assert!(matches!(result, Err(CanonicalError::Empty)));
    }

    #[test]
    fn canonicalize_strips_punctuation() {
        let key = canonicalize("**Hello, World!!").unwrap();
        assert_eq!(key, "hello-world");
    }
}
