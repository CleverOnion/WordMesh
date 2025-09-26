use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;
use validator::{Validate, ValidationError};

pub static USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9_]{3,32}$").expect("username regex must compile"));

#[derive(Debug, Clone, Validate)]
pub struct User {
    pub id: i64,
    #[validate(
        length(min = 3, max = 32),
        custom(function = "validate_username_format")
    )]
    pub username: String,
    pub password_hash: HashedPassword,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HashedPassword(String);

#[derive(Debug, Error)]
pub enum UserDomainError {
    #[error("invalid username: {0}")]
    InvalidUsername(#[from] UsernameValidationError),
    #[error("password hash cannot be empty")]
    EmptyPasswordHash,
}

#[derive(Debug, Error)]
pub enum UsernameValidationError {
    #[error("username must be between 3 and 32 characters")]
    InvalidLength,
    #[error("username contains invalid characters")]
    InvalidFormat,
}

#[derive(Debug, Error)]
pub enum PasswordHashError {
    #[error("password hash cannot be empty")]
    Empty,
}

impl From<PasswordHashError> for UserDomainError {
    fn from(err: PasswordHashError) -> Self {
        match err {
            PasswordHashError::Empty => UserDomainError::EmptyPasswordHash,
        }
    }
}


impl User {
    pub fn new(
        id: i64,
        username: String,
        password_hash: HashedPassword,
        created_at: DateTime<Utc>,
    ) -> Result<Self, UserDomainError> {
        let username = validate_username(username)?;
        Ok(Self {
            id,
            username,
            password_hash,
            created_at,
        })
    }

    #[allow(dead_code)]
    pub fn from_registration(
        username: String,
        password_hash: HashedPassword,
    ) -> Result<Self, UserDomainError> {
        let username = validate_username(username)?;
        Ok(Self {
            id: 0,
            username,
            password_hash,
            created_at: Utc::now(),
        })
    }
}

impl HashedPassword {
    pub fn new(hash: String) -> Result<Self, PasswordHashError> {
        if hash.trim().is_empty() {
            return Err(PasswordHashError::Empty);
        }
        Ok(Self(hash))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_username(username: String) -> Result<String, UsernameValidationError> {
    if username.len() < 3 || username.len() > 32 {
        return Err(UsernameValidationError::InvalidLength);
    }
    if !USERNAME_REGEX.is_match(&username) {
        return Err(UsernameValidationError::InvalidFormat);
    }
    Ok(username)
}

pub fn validate_username_format(username: &str) -> Result<(), ValidationError> {
    if !USERNAME_REGEX.is_match(username) {
        return Err(ValidationError::new("username_format"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_validation_success() {
        let username = "user_123".to_string();
        assert!(validate_username(username).is_ok());
    }

    #[test]
    fn username_validation_failure_length() {
        let username = "ab".to_string();
        assert!(matches!(
            validate_username(username),
            Err(UsernameValidationError::InvalidLength)
        ));
    }

    #[test]
    fn username_validation_failure_format() {
        let username = "invalid-username".to_string();
        assert!(matches!(
            validate_username(username),
            Err(UsernameValidationError::InvalidFormat)
        ));
    }

    #[test]
    fn hashed_password_empty_error() {
        let result = HashedPassword::new("".into());
        assert!(matches!(result, Err(PasswordHashError::Empty)));
    }
}
