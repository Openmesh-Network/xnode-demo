use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Reserve {
    pub xnode_id: String,
}
