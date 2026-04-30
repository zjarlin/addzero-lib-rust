use anyhow::{Context, Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ring::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EncryptedSecret {
    pub key_id: String,
    pub ciphertext: String,
}

#[derive(Clone)]
pub struct SecretCipher {
    key_id: String,
    key: LessSafeKey,
}

impl SecretCipher {
    pub fn from_master_key(master_key: &str) -> Result<Self> {
        let trimmed = master_key.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("ADDZERO_SECRET_MASTER_KEY is empty"));
        }

        let key_bytes = STANDARD
            .decode(trimmed)
            .ok()
            .filter(|bytes| bytes.len() == 32)
            .unwrap_or_else(|| Sha256::digest(trimmed.as_bytes()).to_vec());
        let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| anyhow!("failed to initialize secret cipher"))?;
        Ok(Self {
            key_id: "default".to_string(),
            key: LessSafeKey::new(unbound),
        })
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedSecret> {
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| anyhow!("failed to generate nonce"))?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = plaintext.as_bytes().to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| anyhow!("failed to encrypt secret"))?;
        Ok(EncryptedSecret {
            key_id: self.key_id.clone(),
            ciphertext: format!(
                "v1:{}:{}",
                STANDARD.encode(nonce_bytes),
                STANDARD.encode(in_out)
            ),
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedSecret) -> Result<String> {
        let mut parts = encrypted.ciphertext.split(':');
        let version = parts.next().unwrap_or_default();
        let nonce = parts.next().unwrap_or_default();
        let payload = parts.next().unwrap_or_default();
        if version != "v1" || nonce.is_empty() || payload.is_empty() || parts.next().is_some() {
            return Err(anyhow!("unsupported encrypted secret format"));
        }
        let nonce_bytes = STANDARD.decode(nonce).context("decode nonce")?;
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow!("invalid nonce length"))?;
        let mut in_out = STANDARD.decode(payload).context("decode ciphertext")?;
        let plaintext = self
            .key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_array),
                Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| anyhow!("failed to decrypt secret"))?;
        String::from_utf8(plaintext.to_vec()).context("secret is not utf-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_cipher_should_roundtrip_api_key() {
        let cipher = SecretCipher::from_master_key("local-dev-master-key").unwrap();
        let encrypted = cipher.encrypt("sk-test").unwrap();
        assert_ne!(encrypted.ciphertext, "sk-test");
        assert_eq!(cipher.decrypt(&encrypted).unwrap(), "sk-test");
    }
}
