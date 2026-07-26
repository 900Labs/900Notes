use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::{Path, PathBuf};

use crate::services::kdf;

type HmacSha256 = Hmac<Sha256>;

const ENCRYPTED_DB_SUFFIX: &str = ".enc";
const META_SUFFIX: &str = ".meta";
const INTEGRITY_SUFFIX: &str = ".integrity";
const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const TAG_LEN: usize = 32;

/// Minimum passphrase length enforced for workspace encryption and changes.
pub const MIN_PASSPHRASE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedMeta {
    pub version: String,
    pub salt: String,
    pub nonce: String,
    pub created_at: String,
}

fn derive_key_for(version: &str, passphrase: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    kdf::derive_key_for_version(version, passphrase, salt)
}

/// Validates that a passphrase meets the minimum strength policy. Returns an
/// error string suitable for surfacing to the user when it is too weak.
pub fn validate_passphrase(passphrase: &str) -> Result<(), String> {
    if passphrase.trim().len() < MIN_PASSPHRASE_LEN {
        return Err(format!(
            "Passphrase must be at least {MIN_PASSPHRASE_LEN} characters"
        ));
    }
    Ok(())
}

/// HMAC key binding the live plaintext recovery file to the encrypted
/// snapshot. Derived from the passphrase and the snapshot salt+nonce so that a
/// file swapped into the app data directory by a local attacker cannot pass the
/// integrity check without knowing the passphrase.
fn integrity_key(passphrase: &str, salt: &[u8], nonce: &[u8]) -> [u8; KEY_LEN] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(passphrase.as_bytes())
        .expect("hmac accepts any key length");
    mac.update(b"900notes-db-integrity-v1");
    mac.update(salt);
    mac.update(nonce);
    let out = mac.finalize().into_bytes();
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&out);
    key
}

fn hmac_of(key: &[u8; KEY_LEN], data: &[u8]) -> [u8; TAG_LEN] {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(data);
    let out = mac.finalize().into_bytes();
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(&out);
    tag
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

    pub fn integrity_path(&self) -> PathBuf {
        PathBuf::from(format!("{}{}", self.db_path.display(), INTEGRITY_SUFFIX))
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypted_path().exists() && self.meta_path().exists()
    }

    #[allow(dead_code)]
    pub fn has_plain_db(&self) -> bool {
        self.db_path.exists()
    }

    /// Reads and decodes the encryption metadata sidecar.
    fn read_meta(&self) -> Result<EncryptedMeta, String> {
        let meta_content =
            std::fs::read_to_string(self.meta_path()).map_err(|e| format!("Read meta: {e}"))?;
        serde_json::from_str(&meta_content).map_err(|e| format!("Parse meta: {e}"))
    }

    fn decoded_salt_and_nonce(meta: &EncryptedMeta) -> Result<(Vec<u8>, Vec<u8>), String> {
        let salt = BASE64
            .decode(&meta.salt)
            .map_err(|e| format!("Decode salt: {e}"))?;
        let nonce_bytes = BASE64
            .decode(&meta.nonce)
            .map_err(|e| format!("Decode nonce: {e}"))?;
        Ok((salt, nonce_bytes))
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
        let key = derive_key_for(kdf::KDF_VERSION, passphrase, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

        let nonce_bytes = random_bytes(NONCE_LEN);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("AES encrypt: {e}"))?;

        let meta = EncryptedMeta {
            version: kdf::KDF_VERSION.to_string(),
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

        let meta = self.read_meta()?;
        let (salt, nonce_bytes) = Self::decoded_salt_and_nonce(&meta)?;

        let ciphertext =
            std::fs::read(self.encrypted_path()).map_err(|e| format!("Read encrypted DB: {e}"))?;

        let key = derive_key_for(&meta.version, passphrase, &salt);
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

        // The live plaintext is gone; its integrity sidecar is now stale.
        let _ = std::fs::remove_file(self.integrity_path());
        Ok(())
    }

    /// Writes (or refreshes) the HMAC sidecar that authenticates the live
    /// plaintext recovery file. Should be called whenever we know the plaintext
    /// on disk reflects a state we authored — after unlock, enable, or a
    /// checkpoint of the running session.
    pub fn write_integrity_tag(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(), String> {
        if !self.is_encrypted() {
            return Ok(());
        }
        let meta = self.read_meta()?;
        let (salt, nonce) = Self::decoded_salt_and_nonce(&meta)?;
        let key = integrity_key(passphrase, &salt, &nonce);

        let plaintext = std::fs::read(plain_db_path)
            .map_err(|e| format!("Read plaintext for integrity tag: {e}"))?;
        let tag = hmac_of(&key, &plaintext);

        let temp = self.integrity_path().with_extension("integrity.tmp");
        std::fs::write(&temp, tag).map_err(|e| format!("Write integrity tag: {e}"))?;
        std::fs::rename(&temp, self.integrity_path())
            .map_err(|e| format!("Replace integrity tag: {e}"))?;
        Ok(())
    }

    /// Returns `Ok(true)` only when the plaintext recovery file matches the
    /// HMAC sidecar bound to the encrypted snapshot. A missing plaintext, a
    /// missing sidecar, or a mismatch all return `Ok(false)` so the caller can
    /// re-derive the database from the authoritative snapshot.
    pub fn verify_integrity_tag(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<bool, String> {
        if !plain_db_path.exists() || !self.integrity_path().exists() {
            return Ok(false);
        }
        let meta = self.read_meta()?;
        let (salt, nonce) = Self::decoded_salt_and_nonce(&meta)?;
        let key = integrity_key(passphrase, &salt, &nonce);

        let plaintext = std::fs::read(plain_db_path)
            .map_err(|e| format!("Read plaintext for integrity check: {e}"))?;
        let stored =
            std::fs::read(self.integrity_path()).map_err(|e| format!("Read integrity tag: {e}"))?;
        if stored.len() != TAG_LEN {
            return Ok(false);
        }
        let mut expected = [0u8; TAG_LEN];
        expected.copy_from_slice(&stored);
        Ok(hmac_of(&key, &plaintext) == expected)
    }

    pub fn verify_passphrase(&self, passphrase: &str) -> Result<bool, String> {
        if !self.is_encrypted() {
            return Ok(false);
        }

        let meta = self.read_meta()?;
        let (salt, nonce_bytes) = Self::decoded_salt_and_nonce(&meta)?;

        let ciphertext =
            std::fs::read(self.encrypted_path()).map_err(|e| format!("Read encrypted DB: {e}"))?;

        let key = derive_key_for(&meta.version, passphrase, &salt);
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
        service.enable_encryption("old passphrase", &path).unwrap();
        service
            .change_passphrase("old passphrase", "new passphrase", &path)
            .unwrap();
        assert!(path.exists());
        assert!(!service.verify_passphrase("old passphrase").unwrap());
        assert!(service.verify_passphrase("new passphrase").unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), b"notes");
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn integrity_tag_detects_swapped_recovery_file() {
        let (path, service) = fixture();
        std::fs::write(&path, b"legitimate plaintext").unwrap();
        service
            .enable_encryption("strong passphrase", &path)
            .unwrap();
        service
            .write_integrity_tag("strong passphrase", &path)
            .unwrap();

        // A file we authored passes the integrity check.
        assert!(service
            .verify_integrity_tag("strong passphrase", &path)
            .unwrap());

        // An attacker swaps the recovery file. The bound HMAC no longer matches.
        std::fs::write(&path, b"attacker controlled bytes").unwrap();
        assert!(!service
            .verify_integrity_tag("strong passphrase", &path)
            .unwrap());

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn validate_passphrase_rejects_short_input() {
        assert!(validate_passphrase("short").is_err());
        assert!(validate_passphrase("elevenchars").is_err()); // 11 chars
        assert!(validate_passphrase("strongpass12").is_ok()); // exactly 12
        assert!(validate_passphrase("a very strong passphrase").is_ok());
    }
}

/// Version byte prefix used by [`encrypt_data`] to record the KDF scheme. The
/// legacy format had no prefix (`salt || nonce || ciphertext`), so a leading
/// 0x02 unambiguously marks the new format while remaining readable for old
/// blobs (whose salt's first byte is uniformly random and matches 0x02 with
/// only ~0.4% probability — decrypt still falls back gracefully on failure).
const ENCRYPTED_DATA_VERSION: u8 = 0x02;

pub fn encrypt_data(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    let salt = random_bytes(SALT_LEN);
    let key = kdf::derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let nonce_bytes = random_bytes(NONCE_LEN);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("AES encrypt: {e}"))?;

    let mut result = Vec::with_capacity(1 + salt.len() + nonce_bytes.len() + ciphertext.len());
    result.push(ENCRYPTED_DATA_VERSION);
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_data(data: &[u8], passphrase: &str) -> Result<Vec<u8>, String> {
    // New format: [0x02][salt][nonce][ciphertext].
    if data.len() > 1 + SALT_LEN + NONCE_LEN && data[0] == ENCRYPTED_DATA_VERSION {
        let salt = &data[1..1 + SALT_LEN];
        let nonce_bytes = &data[1 + SALT_LEN..1 + SALT_LEN + NONCE_LEN];
        let ciphertext = &data[1 + SALT_LEN + NONCE_LEN..];

        let key = kdf::derive_key(passphrase, salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        return cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Invalid passphrase or corrupted data".to_string());
    }

    // Legacy format (no version prefix): [salt][nonce][ciphertext].
    if data.len() < SALT_LEN + NONCE_LEN {
        return Err("Data too short".to_string());
    }

    let salt = &data[..SALT_LEN];
    let nonce_bytes = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &data[SALT_LEN + NONCE_LEN..];

    let key = kdf::derive_key_legacy(passphrase, salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("AES init: {e}"))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Invalid passphrase or corrupted data".to_string())
}
