use crate::{OAuth2Error, OAuth2Result};
use ring::rand::{SecureRandom, SystemRandom};

pub(crate) fn random_bytes<const N: usize>() -> OAuth2Result<[u8; N]> {
    let rng = SystemRandom::new();
    let mut bytes = [0_u8; N];
    rng.fill(&mut bytes).map_err(|_| OAuth2Error::Crypto)?;
    Ok(bytes)
}
