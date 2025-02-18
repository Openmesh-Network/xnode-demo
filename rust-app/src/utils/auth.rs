use actix_web::HttpResponse;
use ethsign::Signature;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::utils::error::ResponseError;

use super::{
    keccak::hash_message,
    networking::{request, Request, RequestType},
    wallet::get_signer,
};

#[derive(Serialize, Deserialize, Debug)]
enum LoginMethod {
    WalletSignature { v: u8, r: [u8; 32], s: [u8; 32] },
}

#[derive(Serialize, Deserialize, Debug)]
struct Login {
    pub login_method: LoginMethod,
}

pub fn as_client<T: FnOnce(&Client) -> Option<HttpResponse>>(
    xnode_id: &str,
    action: T,
) -> Option<HttpResponse> {
    let client = match Client::builder()
        .cookie_store(true)
        .timeout(Some(Duration::from_secs(600)))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not build client: {}", e);
            return Some(HttpResponse::InternalServerError().json(ResponseError::new(
                "Networking client could not be created.",
            )));
        }
    };
    let signer = get_signer();
    let message = String::from("Create Xnode Manager session");
    let message_bytes = hash_message(&message);
    let signature: Signature = match signer.sign(&message_bytes) {
        Ok(sig) => sig,
        Err(e) => {
            log::error!("Could not sign login message {}: {}", message, e);
            return Some(
                HttpResponse::InternalServerError()
                    .json(ResponseError::new("New configuration could not be signed.")),
            );
        }
    };

    if let Err(e) = request(
        &client,
        &Request {
            xnode_id: xnode_id.to_string(),
            request_type: RequestType::Post {
                path: String::from("auth/login"),
                body: &Login {
                    login_method: LoginMethod::WalletSignature {
                        v: signature.v,
                        r: signature.r,
                        s: signature.s,
                    },
                },
            },
        },
    ) {
        return Some(e);
    }

    if let Some(e) = action(&client) {
        return Some(e);
    }

    let _logout_result = client.post(format!("{}/auth/logout", xnode_id)).send();

    None
}
