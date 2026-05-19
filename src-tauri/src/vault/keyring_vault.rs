use keyring::Entry;
use zeroize::Zeroizing;
use crate::error::{AppError, AppResult};
use super::VaultBackend;

pub struct KeyringVault;

impl KeyringVault {
    pub fn new() -> Self {
        Self
    }
}

impl VaultBackend for KeyringVault {
    fn store(&self, service: &str, account: &str, secret: &[u8]) -> AppResult<()> {
        let entry = Entry::new(service, account)
            .map_err(|e| AppError::Vault(format!("keyring init: {e}")))?;

        let secret_str = std::str::from_utf8(secret)
            .map_err(|e| AppError::Vault(format!("secret encoding: {e}")))?;

        entry
            .set_password(secret_str)
            .map_err(|e| AppError::Vault(format!("keyring store: {e}")))
    }

    fn retrieve(&self, service: &str, account: &str) -> AppResult<Zeroizing<Vec<u8>>> {
        let entry = Entry::new(service, account)
            .map_err(|e| AppError::Vault(format!("keyring init: {e}")))?;

        let password = entry
            .get_password()
            .map_err(|e| AppError::Vault(format!("keyring retrieve: {e}")))?;

        Ok(Zeroizing::new(password.into_bytes()))
    }

    fn delete(&self, service: &str, account: &str) -> AppResult<()> {
        let entry = Entry::new(service, account)
            .map_err(|e| AppError::Vault(format!("keyring init: {e}")))?;

        entry
            .delete_credential()
            .map_err(|e| AppError::Vault(format!("keyring delete: {e}")))
    }

    fn is_available(&self) -> bool {
        // Prueba creando una entrada temporal — si falla, el keyring no está disponible
        match Entry::new("ia-middleware-probe", "probe") {
            Ok(entry) => {
                let _ = entry.set_password("probe");
                let _ = entry.delete_credential();
                true
            }
            Err(_) => false,
        }
    }
}
