use anyhow::Context;
use ring::rand::{SecureRandom, SystemRandom};

pub(crate) fn random_bytes<const N: usize>() -> anyhow::Result<[u8; N]> {
    let rng = SystemRandom::new();
    let mut bytes = [0_u8; N];
    rng.fill(&mut bytes)
        .map_err(|_| anyhow::anyhow!("crypto random generation failed"))
        .context("failed to generate OAuth2 random bytes")?;
    Ok(bytes)
}
