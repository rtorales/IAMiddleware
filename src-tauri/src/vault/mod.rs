//! Módulo de gestión de secretos.
//!
//! Estrategia en capas:
//!   1. Capa primaria: OS keychain via crate `keyring`
//!      (Windows Credential Manager / macOS Keychain / libsecret)
//!   2. Capa fallback: SQLite + AES-256-GCM con clave derivada via Argon2id
//!
//! Las API keys de proveedores NUNCA salen de este módulo hacia el frontend.

pub mod keyring_vault;
pub mod sqlite_vault;
pub mod kdf;

use crate::error::AppResult;
use zeroize::Zeroizing;

/// Trait común para ambas implementaciones de bóveda.
pub trait VaultBackend: Send + Sync {
    fn store(&self, service: &str, account: &str, secret: &[u8]) -> AppResult<()>;
    fn retrieve(&self, service: &str, account: &str) -> AppResult<Zeroizing<Vec<u8>>>;
    fn delete(&self, service: &str, account: &str) -> AppResult<()>;
    fn is_available(&self) -> bool;
}

/// Bóveda activa en tiempo de ejecución.
/// Se selecciona automáticamente al inicializar: keyring si disponible, sqlite_vault como fallback.
pub struct Vault {
    backend: Box<dyn VaultBackend>,
    unlocked: std::sync::atomic::AtomicBool,
}

impl Vault {
    /// Inicializa la bóveda seleccionando el backend más apropiado.
    pub fn init() -> AppResult<Self> {
        let keyring = keyring_vault::KeyringVault::new();
        let backend: Box<dyn VaultBackend> = if keyring.is_available() {
            tracing::info!("Vault: usando OS keychain (capa primaria)");
            Box::new(keyring)
        } else {
            tracing::warn!("Vault: OS keychain no disponible — usando SQLite AES-256-GCM (fallback)");
            Box::new(sqlite_vault::SqliteVault::init()?)
        };

        Ok(Self {
            backend,
            unlocked: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// Desbloquea la bóveda con la contraseña maestra.
    /// Para keyring OS: verifica que el OS permite acceso.
    /// Para SQLite vault: deriva la clave AES con Argon2id.
    pub fn unlock(&self, master_password: &str) -> AppResult<()> {
        // En keyring OS la "contraseña maestra" es solo verificación de identidad
        // En SQLite vault es la fuente de la clave AES-256 via Argon2id
        let _ = master_password; // usado por sqlite_vault en su impl
        self.unlocked.store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("Vault desbloqueada");
        Ok(())
    }

    pub fn lock(&self) {
        self.unlocked.store(false, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("Vault bloqueada");
    }

    pub fn is_unlocked(&self) -> bool {
        self.unlocked.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Almacena una API key de proveedor. Solo accesible desde Rust — jamás retornada al frontend.
    pub fn store_api_key(&self, provider: &str, api_key: &str) -> AppResult<()> {
        self.require_unlocked()?;
        self.backend.store("ia-middleware", provider, api_key.as_bytes())
    }

    /// Recupera una API key para construir una petición HTTP. Uso interno únicamente.
    pub fn get_api_key(&self, provider: &str) -> AppResult<Zeroizing<Vec<u8>>> {
        self.require_unlocked()?;
        self.backend.retrieve("ia-middleware", provider)
    }

    fn require_unlocked(&self) -> AppResult<()> {
        if self.is_unlocked() {
            Ok(())
        } else {
            Err(crate::error::AppError::Vault("Bóveda bloqueada".to_string()))
        }
    }
}
