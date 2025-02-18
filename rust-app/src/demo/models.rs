use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Reserve {
    pub xnode_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetApp {
    pub xnode_id: String,
    pub flake: String,
}
