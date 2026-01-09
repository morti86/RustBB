use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString
    },
    Argon2,
};

use crate::error::{ForumError, ForumResult};

const MAX_PASSWORD_LENGTH: usize = 64;

pub fn hash(password: impl Into<String>) -> ForumResult<String> {
    let password = password.into();

    if password.is_empty() {
        return Err(ForumError::EmptyPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ForumError::InvalidPassword);
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

    Ok(hashed_password)
}

pub fn compare(password: &str, hashed_password: &str) -> ForumResult<bool> {
    if password.is_empty() {
        return Err(ForumError::InvalidPassword);
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(ForumError::InvalidPassword);
    }

    let parsed_hash = PasswordHash::new(hashed_password)
            .map_err(|_| ForumError::InvalidHashFormat)?;

    let password_matched = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok_and(|_| true);

    Ok(password_matched)
}

