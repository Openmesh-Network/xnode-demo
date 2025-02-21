use log::error;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, remove_file};
use std::time::SystemTime;

use crate::utils::env::{reservationsdir, xnodes};

use super::{auth::as_client, networking};

#[derive(Serialize, Deserialize, Debug)]
pub struct Reservation {
    pub secret: String,
    pub reserved_until: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Xnode {
    pub id: String,
    pub reservation: Option<Reservation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerConfiguration {
    pub flake: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConfigurationAction {
    Set {
        container: String,
        config: ContainerConfiguration,
    },
    Remove {
        container: String,
        backup: bool,
    },
    Update {
        container: String,
        inputs: Vec<String>,
    },
}

pub fn get_xnode(xnode_id: String) -> Xnode {
    let path = reservationsdir().join(path_safe_id(&xnode_id));

    Xnode {
        id: xnode_id.clone(),
        reservation: match read_to_string(&path) {
            Ok(reservation_file) => match serde_json::from_str::<Reservation>(&reservation_file) {
                Ok(reservation) => {
                    if reservation.reserved_until
                        > SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .expect("Invalid system time (duration from unix epoch).")
                            .as_secs()
                    {
                        Some(reservation)
                    } else {
                        if let Err(e) = remove_file(&path) {
                            error!(
                                "Could not remove reservation file {}: {}",
                                path.display(),
                                e
                            );
                        }

                        if let Some(e) = as_client(&xnode_id, |client| {
                            networking::request(
                                client,
                                &networking::Request {
                                    xnode_id: xnode_id.clone(),
                                    request_type: networking::RequestType::Post {
                                        path: String::from("config/change"),
                                        body: vec![ConfigurationAction::Remove {
                                            container: reservation.secret,
                                            backup: false,
                                        }],
                                    },
                                },
                            )
                            .err()
                        }) {
                            error!("Could not clean up demo xnode {}: {:?}", xnode_id, e)
                        }

                        None
                    }
                }
                Err(e) => {
                    error!(
                        "Reservation file {} could not be parsed to expected format: {}. Reservation file contents: {}",
                        path.display(), e, reservation_file
                    );
                    None
                }
            },
            Err(e) => {
                error!("Could not read reservation file {}: {}", path.display(), e);
                None
            }
        },
    }
}

pub fn get_xnodes() -> Vec<Xnode> {
    xnodes().into_iter().map(get_xnode).collect()
}

pub fn path_safe_id(xnode_id: &str) -> String {
    xnode_id.replace(std::path::MAIN_SEPARATOR_STR, "")
}
