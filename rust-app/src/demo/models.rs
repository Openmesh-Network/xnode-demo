use serde::{Deserialize, Serialize};

use crate::utils::networking;

#[derive(Serialize, Deserialize)]
pub struct PublicXnode {
    pub xnode_id: String,
    pub reserved_until: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Reserve {
    pub xnode_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetApp {
    pub xnode_id: String,
    pub secret: String,
    pub flake: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForwardRequest {
    pub secret: String,
    pub request: networking::Request<serde_json::Value>,
}
