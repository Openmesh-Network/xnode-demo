use log::warn;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

use crate::utils::env::{reservationsdir, xnodes};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reservation {
    pub reserved_by: String,
    pub reserved_until: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Xnode {
    pub id: String,
    pub reservation: Option<Reservation>,
}

pub fn get_xnode(xnode_id: String) -> Xnode {
    let path = reservationsdir().join(&xnode_id);

    Xnode {
        id: xnode_id,
        reservation: match read_to_string(&path) {
            Ok(reservation_file) => match serde_json::from_str::<Reservation>(&reservation_file) {
                Ok(reservation) => Some(reservation),
                Err(e) => {
                    warn!(
                        "Reservation file {} could not be parsed to expected format: {}. Reservation file contents: {}",
                        path.display(), e, reservation_file
                    );
                    None
                }
            },
            Err(e) => {
                warn!("Could not read reservation file {}: {}", path.display(), e);
                None
            }
        },
    }
}

pub fn get_xnodes() -> Vec<Xnode> {
    xnodes().into_iter().map(get_xnode).collect()
}
