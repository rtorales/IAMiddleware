//! Derivación de claves con Argon2id.
//! Parámetros: m=65536 KB, t=3 iteraciones, p=4 hilos (mínimo OWASP para almacenamiento offline).

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, Params, PasswordHasher,
};
use zeroize::Zeroizing;
use crate::error::{AppError, AppResult};

/// Longitud de la clave AES-256-GCM derivada.
pub const KEY_LEN: usize = 32;

/// Parámetros Argon2id (OWASP mínimos para almacenamiento de contraseñas)
pub const ARGON2_MEM_KB: u32 = 65_536;   // 64 MB
pub const ARGON2_ITERS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;

/// Resultado de la derivación: clave + salt para almacenamiento.
pub struct DerivedKey {
    pub key: Zeroizing<[u8; KEY_LEN]>,
    pub salt_b64: String,
}

/// Deriva una clave AES-256 desde una contraseña maestra con un salt nuevo (para creación inicial).
pub fn derive_new(password: &str) -> AppResult<DerivedKey> {
    let salt = SaltString::generate(&mut OsRng);
    derive_with_salt(password, salt.as_str())
}

/// Deriva una clave AES-256 desde una contraseña maestra con un salt existente (para verificación).
pub fn derive_with_salt(password: &str, salt_b64: &str) -> AppResult<DerivedKey> {
    let params = Params::new(ARGON2_MEM_KB, ARGON2_ITERS, ARGON2_PARALLELISM, Some(KEY_LEN))
        .map_err(|e| AppError::Vault(format!("argon2 params: {e}")))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let salt = argon2::password_hash::SaltString::from_b64(salt_b64)
        .map_err(|e| AppError::Vault(format!("invalid salt: {e}")))?;

    let mut key_bytes = Zeroizing::new([0u8; KEY_LEN]);

    argon2
        .hash_password_into(password.as_bytes(), salt.as_str().as_bytes(), key_bytes.as_mut())
        .map_err(|e| AppError::Vault(format!("argon2 hash: {e}")))?;

    Ok(DerivedKey {
        key: key_bytes,
        salt_b64: salt_b64.to_string(),
    })
}
