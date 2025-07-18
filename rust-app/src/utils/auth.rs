use hex::ToHex;
use xnode_manager_sdk::utils::Session;

use crate::utils::{error::Error, time::get_time};

use super::{keccak::hash_message, wallet::get_signer};

pub async fn get_session(xnode_id: &str) -> Result<Session, Error> {
    let signer = get_signer();

    let addr: String = signer.public().address().encode_hex();
    let user = format!("eth:{addr}");
    let domain = xnode_id.to_string();
    let timestamp = get_time();
    let message = format!("Xnode Auth authenticate {domain} at {timestamp}");
    let message_bytes = hash_message(&message);
    let signature = match signer.sign(&message_bytes) {
        Ok(sig) => {
            let bytes: Vec<u8> = sig.r.into_iter().chain(sig.s).chain([sig.v]).collect();
            let hex: String = bytes.encode_hex();

            format!("0x{hex}")
        }
        Err(e) => {
            return Err(Error::EthSignError(ethsign::Error::Secp256k1(e)));
        }
    };

    xnode_manager_sdk::auth::login(xnode_manager_sdk::auth::LoginInput {
        base_url: format!("https://{domain}"),
        user: xnode_manager_sdk::auth::User::with_signature(user, signature, timestamp.to_string()),
    })
    .await
    .map_err(Error::XnodeManagerSDKError)
}
