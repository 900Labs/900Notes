use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const ENCRYPTED_DB_SUFFIX: &str = ".enc";
const META_SUFFIX: &str = ".meta";
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedMeta {
    pub version: String,
    pub salt: String,
    pub nonce: String,
    pub created_at: String,
}

fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut current = [0u8; KEY_LEN];
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(salt);
    current.copy_from_slice(&hasher.finalize());

    for _ in 0..100_000 {
        let mut h = Sha256::new();
        h.update(current);
        current.copy_from_slice(&h.finalize());
    }
    current
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    getrandom::getrandom(&mut buf).expect("CSPRNG failed");
    buf
}

pub struct EncryptionService {
    db_path: PathBuf,
}

impl EncryptionService {
    pub fn new(db_path: &Path) -> Self {
        EncryptionService {
            db_path: db_path.to_path_buf(),
        }
    }

    pub fn encrypted_path(&self) -> PathBuf {
        PathBuf::from(format!("{}{}", self.db_path.display(), ENCRYPTED_DB_SUFFIX))
    }

    pub fn meta_path(&self) -> PathBuf {
        PathBuf::from(format!("{}{}", self.db_path.display(), META_SUFFIX))
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypted_path().exists() && self.meta_path().exists()
    }

    #[allow(dead_code)]
    pub fn has_plain_db(&self) -> bool {
        self.db_path.exists()
    }

    pub fn enable_encryption(&self, passphrase: &str, plain_db_path: &Path) -> Result<(), String> {
        if self.is_encrypted() {
            return Err("Encryption is already enabled".to_string());
        }

        self.write_encrypted_snapshot(passphrase, plain_db_path)?;
        Ok(())
    }

    fn encrypted_payload(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(Vec<u8>, String), String> {
        let plaintext = std::fs::read(plain_db_path).map_err(|e| format!("Read database: {e}"))?;

        let salt = random_bytes(SALT_LEN);
        let key = derive_key(passphrase, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

        let nonce_bytes = random_bytes(NONCE_LEN);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("AES encrypt: {e}"))?;

        let meta = EncryptedMeta {
            version: "1.0.0".to_string(),
            salt: BASE64.encode(&salt),
            nonce: BASE64.encode(&nonce_bytes),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let meta_json =
            serde_json::to_string_pretty(&meta).map_err(|e| format!("Serialize meta: {e}"))?;

        Ok((ciphertext, meta_json))
    }

    fn write_encrypted_snapshot(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(), String> {
        let (ciphertext, meta_json) = self.encrypted_payload(passphrase, plain_db_path)?;
        let encrypted_path = self.encrypted_path();
        let meta_path = self.meta_path();
        let encrypted_temp = encrypted_path.with_extension("enc.tmp");
        let meta_temp = meta_path.with_extension("meta.tmp");
        std::fs::write(&encrypted_temp, ciphertext)
            .map_err(|e| format!("Write encrypted temporary file: {e}"))?;
        if let Err(error) = std::fs::write(&meta_temp, meta_json) {
            let _ = std::fs::remove_file(&encrypted_temp);
            return Err(format!("Write metadata temporary file: {error}"));
        }
        let encrypted_backup = encrypted_path.with_extension("enc.previous");
        let meta_backup = meta_path.with_extension("meta.previous");
        for backup in [&encrypted_backup, &meta_backup] {
            if backup.exists() {
                let _ = std::fs::remove_file(backup);
            }
        }
        if encrypted_path.exists() {
            std::fs::rename(&encrypted_path, &encrypted_backup)
                .map_err(|e| format!("Stage encrypted DB replacement: {e}"))?;
        }
        if meta_path.exists() {
            if let Err(error) = std::fs::rename(&meta_path, &meta_backup) {
                let _ = std::fs::rename(&encrypted_backup, &encrypted_path);
                return Err(format!("Stage encryption metadata replacement: {error}"));
            }
        }
        let replace_result = (|| {
            std::fs::rename(&encrypted_temp, &encrypted_path)
                .map_err(|e| format!("Replace encrypted DB: {e}"))?;
            std::fs::rename(&meta_temp, &meta_path)
                .map_err(|e| format!("Replace encryption metadata: {e}"))?;
            Ok::<(), String>(())
        })();
        if let Err(error) = replace_result {
            let _ = std::fs::remove_file(&encrypted_path);
            let _ = std::fs::remove_file(&meta_path);
            if encrypted_backup.exists() {
                let _ = std::fs::rename(&encrypted_backup, &encrypted_path);
            }
            if meta_backup.exists() {
                let _ = std::fs::rename(&meta_backup, &meta_path);
            }
            let _ = std::fs::remove_file(&encrypted_temp);
            let _ = std::fs::remove_file(&meta_temp);
            return Err(error);
        }
        let _ = std::fs::remove_file(encrypted_backup);
        let _ = std::fs::remove_file(meta_backup);
        Ok(())
    }

    pub fn decrypt_to_path(&self, passphrase: &str, output_path: &Path) -> Result<(), String> {
        if !self.is_encrypted() {
            return Err("Encryption is not enabled".to_string());
        }

        let meta_content =
            std::fs::read_to_string(self.meta_path()).map_err(|e| format!("Read meta: {e}"))?;
        let meta: EncryptedMeta =
            serde_json::from_str(&meta_content).map_err(|e| format!("Parse meta: {e}"))?;

        let salt = BASE64
            .decode(&meta.salt)
            .map_err(|e| format!("Decode salt: {e}"))?;
        let nonce_bytes = BASE64
            .decode(&meta.nonce)
            .map_err(|e| format!("Decode nonce: {e}"))?;

        let ciphertext =
            std::fs::read(self.encrypted_path()).map_err(|e| format!("Read encrypted DB: {e}"))?;

        let key = derive_key(passphrase, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Invalid passphrase or corrupted database".to_string())?;

        let temp = output_path.with_extension("db.decrypting");
        std::fs::write(&temp, &plaintext).map_err(|e| format!("Write decrypted DB: {e}"))?;
        std::fs::rename(&temp, output_path).map_err(|e| format!("Replace decrypted DB: {e}"))?;

        Ok(())
    }

    pub fn disable_encryption(
        &self,
        passphrase: &str,
        output_db_path: &Path,
    ) -> Result<(), String> {
        if output_db_path.exists() {
            if !self.verify_passphrase(passphrase)? {
                return Err("Invalid passphrase".to_string());
            }
        } else {
            self.decrypt_to_path(passphrase, output_db_path)?;
        }

        std::fs::remove_file(self.encrypted_path())
            .map_err(|e| format!("Remove encrypted DB: {e}"))?;
        std::fs::remove_file(self.meta_path()).map_err(|e| format!("Remove meta: {e}"))?;

        Ok(())
    }

    pub fn change_passphrase(
        &self,
        old_passphrase: &str,
        new_passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(), String> {
        if !self.verify_passphrase(old_passphrase)? {
            return Err("Invalid current passphrase".to_string());
        }
        let temp = self.db_path.with_extension("db.passphrase-change");
        let source = if plain_db_path.exists() {
            plain_db_path
        } else {
            self.decrypt_to_path(old_passphrase, &temp)?;
            temp.as_path()
        };
        let result = self.write_encrypted_snapshot(new_passphrase, source);
        if temp.exists() {
            let _ = std::fs::remove_file(temp);
        }
        result
    }

    pub fn re_encrypt_on_shutdown(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(), String> {
        if !self.is_encrypted() {
            return Ok(());
        }

        self.write_encrypted_snapshot(passphrase, plain_db_path)?;
        std::fs::remove_file(plain_db_path).map_err(|e| format!("Remove plaintext DB: {e}"))?;

        Ok(())
    }

    pub fn verify_passphrase(&self, passphrase: &str) -> Result<bool, String> {
        if !self.is_encrypted() {
            return Ok(false);
        }

        let meta_content =
            std::fs::read_to_string(self.meta_path()).map_err(|e| format!("Read meta: {e}"))?;
        let meta: EncryptedMeta =
            serde_json::from_str(&meta_content).map_err(|e| format!("Parse meta: {e}"))?;

        let salt = BASE64
            .decode(&meta.salt)
            .map_err(|e| format!("Decode salt: {e}"))?;
        let nonce_bytes = BASE64
            .decode(&meta.nonce)
            .map_err(|e| format!("Decode nonce: {e}"))?;

        let ciphertext =
            std::fs::read(self.encrypted_path()).map_err(|e| format!("Read encrypted DB: {e}"))?;

        let key = derive_key(passphrase, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        match cipher.decrypt(nonce, ciphertext.as_ref()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (PathBuf, EncryptionService) {
        let root =
            std::env::temp_dir().join(format!("900notes-encryption-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("workspace.db");
        (path.clone(), EncryptionService::new(&path))
    }

    #[test]
    fn enable_keeps_live_plaintext_and_writes_recoverable_snapshot() {
        let (path, service) = fixture();
        std::fs::write(&path, b"first").unwrap();
        service.enable_encryption("old", &path).unwrap();
        assert!(path.exists());
        std::fs::write(&path, b"newer unsaved session").unwrap();
        assert!(service.verify_passphrase("old").unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), b"newer unsaved session");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn clean_shutdown_refreshes_snapshot_and_removes_plaintext() {
        let (path, service) = fixture();
        std::fs::write(&path, b"first").unwrap();
        service.enable_encryption("secret", &path).unwrap();
        std::fs::write(&path, b"latest").unwrap();
        service.re_encrypt_on_shutdown("secret", &path).unwrap();
        assert!(!path.exists());
        service.decrypt_to_path("secret", &path).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"latest");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn change_passphrase_preserves_the_live_database() {
        let (path, service) = fixture();
        std::fs::write(&path, b"notes").unwrap();
        service.enable_encryption("old", &path).unwrap();
        service.change_passphrase("old", "new", &path).unwrap();
        assert!(path.exists());
        assert!(!service.verify_passphrase("old").unwrap());
        assert!(service.verify_passphrase("new").unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), b"notes");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}

pub fn encrypt_data(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let salt = random_bytes(SALT_LEN);
    let key = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let nonce_bytes = random_bytes(NONCE_LEN);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("AES encrypt: {e}"))?;

    let mut result = Vec::with_capacity(salt.len() + nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_data(data: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err("Data too short".to_string());
    }

    let salt = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];

    let key = derive_key(passphrase, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Invalid passphrase or corrupted data".to_string())
}
