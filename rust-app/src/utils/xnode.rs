use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, remove_file};
use xnode_controller::XnodeController;

use crate::utils::auth::get_session;
use crate::utils::env::{reservationsdir, xnodes};
use crate::utils::time::get_time;

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

pub async fn get_xnode(xnode_id: String) -> Xnode {
    let path = reservationsdir().join(path_safe_id(&xnode_id));

    Xnode {
        id: xnode_id.clone(),
        reservation: match read_to_string(&path) {
            Ok(reservation_file) => match serde_json::from_str::<Reservation>(&reservation_file) {
                Ok(reservation) => {
                    if reservation.reserved_until > get_time() {
                        Some(reservation)
                    } else {
                        if let Err(e) = remove_file(&path) {
                            log::error!(
                                "Could not remove reservation file {path}: {e}",
                                path = path.display()
                            );
                        }

                        clean_xnode(&xnode_id).await;

                        None
                    }
                }
                Err(e) => {
                    log::error!(
                        "Reservation file {path} could not be parsed to expected format: {e}. Reservation file contents: {reservation_file}",
                        path = path.display(),
                    );
                    None
                }
            },
            Err(_e) => None,
        },
    }
}

pub async fn get_xnodes() -> Vec<Xnode> {
    join_all(xnodes().into_iter().map(get_xnode)).await
}

pub fn path_safe_id(xnode_id: &str) -> String {
    xnode_id.replace(std::path::MAIN_SEPARATOR_STR, "")
}

pub async fn clean_xnode(xnode_id: &str) {
    match get_session(xnode_id).await {
        Ok(session) => {
            let controller = ControlledXnode { session };
            if let Err(e) = controller.set_controller(None).await {
                log::error!("Could not remove controller from {xnode_id}: {e:?}");
            }

            let containers = match xnode_manager_sdk::config::containers(
                xnode_manager_sdk::config::ContainersInput::new(controller.get_session()),
            )
            .await
            {
                Ok(containers) => containers,
                Err(e) => {
                    log::error!("Could not get containers from {xnode_id}: {e:?}");
                    vec![]
                }
            };

            join_all(containers.into_iter().map(async |container| {
                if let Err(e) = xnode_manager_sdk::config::remove(
                    xnode_manager_sdk::config::RemoveInput::new_with_path(
                        controller.get_session(),
                        xnode_manager_sdk::config::RemovePath {
                            container: container.clone(),
                        },
                    ),
                )
                .await
                {
                    log::error!("Could not remove container {container} from {xnode_id}: {e:?}");
                }
            }))
            .await;
        }
        Err(e) => {
            log::error!("Could not get Xnode session for {xnode_id}: {e:?}");
        }
    }
}

pub struct ControlledXnode {
    pub session: xnode_manager_sdk::utils::Session,
}

impl XnodeController for ControlledXnode {
    fn get_session(&self) -> &xnode_manager_sdk::utils::Session {
        &self.session
    }

    async fn check_controller(&self) -> Option<String> {
        get_xnode(self.session.base_url.clone())
            .await
            .reservation
            .map(|r| r.reserved_by)
    }

    fn controller_config(&self, controller: String) -> String {
        let manager = self.session.base_url.replace("https://", "");
        let app = self.session.base_url.replace("https://manager.", "");
        format!(
            "\
services.xnode-auth.domains.\"{manager}\".accessList.\"{controller}\" = {{ paths = \"^(?:\\/config.*|\\/file\\/container:.*|\\/info.*|\\/process\\/container:.*|\\/usage.*|\\/request.*)\"; }};
services.xnode-auth.domains.\"{app}\".accessList.\"{controller}\" = {{ }};\
"
        )
    }
}
