use ethsign::SecretKey;
use log::{error, info, warn};
use rand::{rng, Rng};
use std::fs::{read, write};

use super::env::datadir;

pub fn get_signer() -> SecretKey {
    let path = datadir().join("secret.key");
    let key = match read(&path) {
        Ok(secret) => secret
            .try_into()
            .inspect_err(|e| {
                error!(
                    "Private key {} in incorrect format: {:?}",
                    path.display(),
                    e
                );
            })
            .ok()
            .unwrap_or_else(|| generate_private_key()),
        Err(e) => {
            warn!("Could not read private key {}: {}", path.display(), e);

            generate_private_key()
        }
    };

    SecretKey::from_raw(&key).unwrap_or_else(|e| {
        panic!("Could not convert private key into SecretKey: {}", e);
    })
}

fn generate_private_key() -> [u8; 32] {
    info!("Generating new secret key");
    let priv_key = random_bytes();

    let path = datadir().join("secret.key");
    if let Err(e) = write(&path, &priv_key) {
        error!("Could not save private key {}: {}", path.display(), e);
    }

    priv_key
}

fn random_bytes() -> [u8; 32] {
    let mut secret = [0u8; 32];
    rng().fill(&mut secret);
    secret
}
