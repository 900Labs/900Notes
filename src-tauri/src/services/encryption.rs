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

        std::fs::write(self.encrypted_path(), &ciphertext)
            .map_err(|e| format!("Write encrypted DB: {e}"))?;
        std::fs::write(self.meta_path(), meta_json).map_err(|e| format!("Write meta: {e}"))?;

        std::fs::remove_file(plain_db_path).map_err(|e| format!("Remove plain DB: {e}"))?;

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

        std::fs::write(output_path, &plaintext).map_err(|e| format!("Write decrypted DB: {e}"))?;

        Ok(())
    }

    pub fn disable_encryption(
        &self,
        passphrase: &str,
        output_db_path: &Path,
    ) -> Result<(), String> {
        self.decrypt_to_path(passphrase, output_db_path)?;

        std::fs::remove_file(self.encrypted_path())
            .map_err(|e| format!("Remove encrypted DB: {e}"))?;
        std::fs::remove_file(self.meta_path()).map_err(|e| format!("Remove meta: {e}"))?;

        Ok(())
    }

    pub fn change_passphrase(
        &self,
        old_passphrase: &str,
        new_passphrase: &str,
        temp_db_path: &Path,
    ) -> Result<(), String> {
        self.decrypt_to_path(old_passphrase, temp_db_path)?;

        std::fs::remove_file(self.encrypted_path())
            .map_err(|e| format!("Remove old encrypted DB: {e}"))?;
        std::fs::remove_file(self.meta_path()).map_err(|e| format!("Remove old meta: {e}"))?;

        self.enable_encryption(new_passphrase, temp_db_path)?;

        std::fs::remove_file(temp_db_path).map_err(|e| format!("Remove temp DB: {e}"))?;

        Ok(())
    }

    pub fn re_encrypt_on_shutdown(
        &self,
        passphrase: &str,
        plain_db_path: &Path,
    ) -> Result<(), String> {
        if !self.is_encrypted() {
            return Ok(());
        }

        let plaintext =
            std::fs::read(plain_db_path).map_err(|e| format!("Read plaintext DB: {e}"))?;

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

        std::fs::write(self.encrypted_path(), &ciphertext)
            .map_err(|e| format!("Write encrypted DB: {e}"))?;
        std::fs::write(self.meta_path(), meta_json).map_err(|e| format!("Write meta: {e}"))?;

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
