//! Password hashing and verification utilities.

use bcrypt::{DEFAULT_COST, hash, verify};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password hash failed: {0}")]
    Hash(#[from] bcrypt::BcryptError),
    #[error("password verification failed")]
    Verify,
    #[error("password is empty")]
    Empty,
}

pub fn hash_password(raw: &str, cost: u32) -> Result<String, PasswordError> {
    if raw.trim().is_empty() {
        return Err(PasswordError::Empty);
    }
    let effective_cost = if cost < 4 { DEFAULT_COST } else { cost };
    Ok(hash(raw, effective_cost)?)
}

pub fn verify_password(raw: &str, hashed: &str) -> Result<bool, PasswordError> {
    if raw.trim().is_empty() || hashed.trim().is_empty() {
        return Err(PasswordError::Empty);
    }
    verify(raw, hashed).map_err(PasswordError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_password_success() {
        let hashed = hash_password("secret", 10).expect("hash");
        assert!(verify_password("secret", &hashed).unwrap());
        assert!(!verify_password("wrong", &hashed).unwrap());
    }

    #[test]
    fn hash_password_empty() {
        let result = hash_password("", 10);
        assert!(matches!(result, Err(PasswordError::Empty)));
    }

    #[test]
    fn verify_password_empty_inputs() {
        let hashed = hash_password("secret", 10).expect("hash");
        assert!(matches!(
            verify_password("", &hashed),
            Err(PasswordError::Empty)
        ));
        assert!(matches!(
            verify_password("secret", ""),
            Err(PasswordError::Empty)
        ));
    }
}
