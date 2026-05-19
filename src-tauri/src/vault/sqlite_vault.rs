//! Bóveda fallback: SQLite + AES-256-GCM.
//!
//! Cada secreto se encripta individualmente con un nonce aleatorio de 12 bytes.
//! La clave AES-256 se deriva de la contraseña maestra via Argon2id (kdf.rs).
//! El salt de Argon2id se almacena en la tabla `vault_meta` en texto plano (es público por diseño).

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng as AeadOsRng},
    Aes256Gcm, Key, Nonce,
};
use rusqlite::{Connection, params};
use std::sync::Mutex;
use zeroize::Zeroizing;

use super::VaultBackend;
use crate::error::{AppError, AppResult};

pub struct SqliteVault {
    conn: Mutex<Connection>,
    cipher: Mutex<Option<Aes256Gcm>>,
}

impl SqliteVault {
    pub fn init() -> AppResult<Self> {
        let db_path = vault_db_path()?;
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS vault_meta (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS vault_entries (
                 service  TEXT NOT NULL,
                 account  TEXT NOT NULL,
                 ciphertext BLOB NOT NULL,
                 nonce    BLOB NOT NULL,
                 PRIMARY KEY (service, account)
             );",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
            cipher: Mutex::new(None),
        })
    }

    /// Inicializa el cipher AES tras derivar la clave con la contraseña maestra.
    pub fn unlock_with_password(&self, password: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();

        let salt_b64: Option<String> = conn
            .query_row(
                "SELECT value FROM vault_meta WHERE key = 'argon2_salt'",
                [],
                |row| row.get(0),
            )
            .ok();

        let derived = match salt_b64 {
            Some(salt) => super::kdf::derive_with_salt(password, &salt)?,
            None => {
                // Primera vez: generar salt y almacenarlo
                let dk = super::kdf::derive_new(password)?;
                conn.execute(
                    "INSERT OR REPLACE INTO vault_meta (key, value) VALUES ('argon2_salt', ?1)",
                    params![dk.salt_b64],
                )?;
                dk
            }
        };

        let aes_key = Key::<Aes256Gcm>::from_slice(derived.key.as_ref());
        let cipher = Aes256Gcm::new(aes_key);

        *self.cipher.lock().unwrap() = Some(cipher);
        Ok(())
    }

    fn get_cipher(&self) -> AppResult<std::sync::MutexGuard<'_, Option<Aes256Gcm>>> {
        let guard = self.cipher.lock().unwrap();
        if guard.is_none() {
            return Err(AppError::Vault("SQLite vault no desbloqueada".to_string()));
        }
        Ok(guard)
    }
}

impl VaultBackend for SqliteVault {
    fn store(&self, service: &str, account: &str, secret: &[u8]) -> AppResult<()> {
        let guard = self.get_cipher()?;
        let cipher = guard.as_ref().unwrap();

        let nonce = Aes256Gcm::generate_nonce(&mut AeadOsRng);
        let ciphertext = cipher
            .encrypt(&nonce, secret)
            .map_err(|e| AppError::Vault(format!("encrypt: {e}")))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO vault_entries (service, account, ciphertext, nonce)
             VALUES (?1, ?2, ?3, ?4)",
            params![service, account, ciphertext, nonce.as_slice()],
        )?;
        Ok(())
    }

    fn retrieve(&self, service: &str, account: &str) -> AppResult<Zeroizing<Vec<u8>>> {
        let guard = self.get_cipher()?;
        let cipher = guard.as_ref().unwrap();

        let conn = self.conn.lock().unwrap();
        let (ciphertext, nonce_bytes): (Vec<u8>, Vec<u8>) = conn
            .query_row(
                "SELECT ciphertext, nonce FROM vault_entries WHERE service = ?1 AND account = ?2",
                params![service, account],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| AppError::Vault(format!("key not found: {service}/{account}")))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| AppError::Vault(format!("decrypt: {e}")))?;

        Ok(Zeroizing::new(plaintext))
    }

    fn delete(&self, service: &str, account: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM vault_entries WHERE service = ?1 AND account = ?2",
            params![service, account],
        )?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        // SQLite vault siempre está disponible como fallback
        true
    }
}

fn vault_db_path() -> AppResult<std::path::PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| AppError::Vault("No se pudo determinar data_dir".to_string()))?;
    let dir = base.join("ia-middleware");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("vault.db"))
}
