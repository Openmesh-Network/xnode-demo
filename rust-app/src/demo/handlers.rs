use actix_web::{HttpResponse, Responder, dev::ConnectionInfo, get, post, web};
use hex::ToHex;
use std::fs::write;
use xnode_controller::XnodeController;

use crate::{
    demo::models::Reserve,
    utils::{
        auth::get_session,
        env::{self, reservationduration, reservationsdir},
        error::ResponseError,
        time::get_time,
        wallet::get_signer,
        xnode::{ControlledXnode, Reservation, get_xnode, get_xnodes, path_safe_id},
    },
};

#[get("/user")]
async fn user() -> impl Responder {
    let addr: String = get_signer().public().address().encode_hex();
    HttpResponse::Ok().json(format!("eth:{addr}"))
}

#[get("/xnodes")]
async fn xnodes() -> impl Responder {
    HttpResponse::Ok().json(get_xnodes().await)
}

#[post("/reserve")]
async fn reserve(reserve: web::Json<Reserve>, conn: ConnectionInfo) -> impl Responder {
    let xnode_id = reserve.xnode_id.clone();
    if !env::xnodes().contains(&xnode_id) {
        return HttpResponse::BadRequest().json(ResponseError::new("Invalid Xnode id."));
    }

    let xnode = get_xnode(xnode_id).await;
    let xnode_id = &xnode.id;
    if xnode.reservation.is_some() {
        return HttpResponse::BadRequest().json(ResponseError::new("Xnode is already reserved."));
    }

    let reserved_by = match conn.realip_remote_addr() {
        Some(addr) => format!("ip:{addr}"),
        None => {
            return HttpResponse::BadRequest().json(ResponseError::new("No IP set on connection."));
        }
    };
    let reserved_until = get_time() + reservationduration();

    let session = match get_session(xnode_id).await {
        Ok(session) => session,
        Err(e) => {
            log::error!("Could not create Xnode session on {xnode_id}: {e:?}");
            return HttpResponse::InternalServerError()
                .json(ResponseError::new("Xnode could not be reserved."));
        }
    };
    let controller = ControlledXnode { session };
    if let Err(e) = controller.set_controller(Some(reserved_by.clone())).await {
        log::error!("Could not update controller to {reserved_by} on {xnode_id}: {e:?}");
        return HttpResponse::InternalServerError()
            .json(ResponseError::new("Xnode could not be reserved."));
    };

    let reservation = Reservation {
        reserved_by,
        reserved_until,
    };
    let reservation_json: String = match serde_json::to_string(&reservation) {
        Ok(json) => json,
        Err(e) => {
            log::error!(
                "Could not convert reservations to json: {e}. Reservation: {reservation:?}",
            );
            return HttpResponse::InternalServerError()
                .json(ResponseError::new("Xnode could not be reserved."));
        }
    };

    let path = reservationsdir().join(path_safe_id(xnode_id));
    if let Err(e) = write(&path, &reservation_json) {
        log::error!(
            "Could not write reservations file at {path}: {e}. Reservation json: {reservation_json}",
            path = path.display(),
        );
        return HttpResponse::InternalServerError()
            .json(ResponseError::new("Xnode could not be reserved."));
    }

    HttpResponse::Ok().json(get_xnode(xnode.id).await)
}
