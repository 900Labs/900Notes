//! Password-based key derivation for 900Notes.
//!
//! Workspace encryption, encrypted share bundles, and the LAN sync transport
//! all derive their AES-256 keys from a user-supplied passphrase. This module
//! provides a memory-hard [Argon2id] KDF as the current scheme and retains the
//! legacy iterative-SHA-256 derivation solely so databases, bundles, and
//! messages written before the upgrade can still be read and migrated.
//!
//! The active version is `"2.0.0"`. Writers always emit the current scheme;
//! readers branch on the stored `version` so existing data remains accessible
//! and a `change_passphrase` / re-export transparently upgrades it.
//!
//! [Argon2id]: https://datatracker.ietf.org/doc/html/rfc9106

use argon2::{
    password_hash::{PasswordHasher, Salt, SaltString},
    Algorithm, Argon2, Params, Version,
};
use sha2::{Digest, Sha256};

/// KDF version written by all new encrypting code paths.
pub const KDF_VERSION: &str = "2.0.0";

/// Legacy KDF version (iterative SHA-256). Kept for read-only backward
/// compatibility; no new data is written with this version.
pub const LEGACY_KDF_VERSION: &str = "1.0.0";

const LEGACY_ROUNDS: u32 = 100_000;

/// Memory-hard Argon2id derivation used for all newly written encrypted data.
///
/// `salt` must be the raw (non-base64) salt bytes. The Argon2 salt string is
/// base64-encoded internally per the PHC spec; we pass the raw bytes through
/// `SaltString::encode_b64` so the output is deterministic for a given
/// passphrase + salt pair.
pub fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let params = Params::new(64 * 1024, 3, 4, Some(32)).expect("valid argon2 params");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let salt_string = SaltString::encode_b64(salt).expect("salt fits argon2 encoding");
    let salt: Salt = Salt::from(&salt_string);

    let hash = argon2
        .hash_password(passphrase.as_bytes(), salt)
        .expect("argon2 derivation does not fail for valid params");
    let output = hash.hash.expect("32-byte hash present");
    let mut key = [0u8; 32];
    key.copy_from_slice(output.as_ref());
    key
}

/// Legacy iterative-SHA-256 derivation. Only used to read data written with
/// [`LEGACY_KDF_VERSION`].
pub fn derive_key_legacy(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut current = [0u8; 32];
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(salt);
    current.copy_from_slice(&hasher.finalize());

    for _ in 0..LEGACY_ROUNDS {
        let mut h = Sha256::new();
        h.update(current);
        current.copy_from_slice(&h.finalize());
    }
    current
}

/// Selects the derivation function for a stored `version` string.
pub fn derive_key_for_version(version: &str, passphrase: &str, salt: &[u8]) -> [u8; 32] {
    if version == LEGACY_KDF_VERSION {
        derive_key_legacy(passphrase, salt)
    } else {
        derive_key(passphrase, salt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argon2_is_deterministic_for_same_inputs() {
        let a = derive_key("correct horse battery staple", b"abcdef0123456789abcdef0123456789");
        let b = derive_key("correct horse battery staple", b"abcdef0123456789abcdef0123456789");
        assert_eq!(a, b);
    }

    #[test]
    fn argon2_changes_with_salt() {
        let a = derive_key("passphrase", b"salt-number-one-16bytes");
        let b = derive_key("passphrase", b"salt-number-two-16bytes");
        assert_ne!(a, b);
    }

    #[test]
    fn argon2_changes_with_passphrase() {
        let a = derive_key("passphrase-a", b"same-salt-bytes-here!");
        let b = derive_key("passphrase-b", b"same-salt-bytes-here!");
        assert_ne!(a, b);
    }

    #[test]
    fn legacy_branch_still_derives() {
        let salt = b"saltsaltsaltsalt";
        let key = derive_key_for_version(LEGACY_KDF_VERSION, "passphrase", salt);
        assert_eq!(key.len(), 32);
        // Distinct from the new scheme for the same inputs.
        assert_ne!(key, derive_key("passphrase", salt));
    }

    #[test]
    fn unknown_version_defaults_to_current() {
        let salt = b"saltsaltsaltsalt";
        let key = derive_key_for_version("9.9.9", "passphrase", salt);
        assert_eq!(key, derive_key("passphrase", salt));
    }
}
