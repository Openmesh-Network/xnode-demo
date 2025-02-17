use actix_web::{
    get, post,
    web::{self, Path},
    HttpRequest, HttpResponse, Responder,
};
use hex::ToHex;
use std::fs::write;
use std::time::SystemTime;

use crate::utils::{
    auth::as_client,
    env::{self, reservationduration, reservationsdir},
    error::ResponseError,
    networking,
    wallet::get_signer,
    xnode::{get_xnode, get_xnodes, Reservation},
};

#[get("/address")]
async fn address() -> impl Responder {
    let signer = get_signer();
    let address: String = signer.public().address().encode_hex();

    HttpResponse::Ok().json(address)
}

#[get("/xnodes")]
async fn xnodes() -> impl Responder {
    HttpResponse::Ok().json(get_xnodes())
}

#[post("/reserve/{xnode}")]
async fn reserve(path: Path<String>, request: HttpRequest) -> impl Responder {
    let xnode_id = path.into_inner();
    if !env::xnodes().contains(&xnode_id) {
        return HttpResponse::BadRequest().json(ResponseError::new("Invalid Xnode id."));
    }

    let xnode = get_xnode(xnode_id);
    if xnode.reservation.is_some() {
        return HttpResponse::BadRequest().json(ResponseError::new("Xnode is already reserved."));
    }

    let reserved_by: String;
    if let Some(addr) = request.peer_addr() {
        reserved_by = format!("{}", addr.ip());
    } else {
        return HttpResponse::BadRequest()
            .json(ResponseError::new("IP address on connection is not set."));
    }

    let system_time = SystemTime::now();
    let reserved_until = match system_time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() + reservationduration(),
        Err(e) => {
            log::error!(
                "Could not convert system time to epoch: {}. System time: {:?}",
                e,
                system_time
            );
            return HttpResponse::InternalServerError()
                .json(ResponseError::new("Xnode could not be reserved."));
        }
    };

    let reservation = Reservation {
        reserved_by,
        reserved_until,
    };
    let reservation_json: String = match serde_json::to_string(&reservation) {
        Ok(json) => json,
        Err(e) => {
            log::error!(
                "Could not convert reservations to json: {}. Reservation: {:?}",
                e,
                reservation
            );
            return HttpResponse::InternalServerError()
                .json(ResponseError::new("Xnode could not be reserved."));
        }
    };

    let path = reservationsdir().join(&xnode.id);
    if let Err(e) = write(&path, &reservation_json) {
        log::error!(
            "Could not write reservations file at {}: {}. Reservation json: {}",
            path.display(),
            e,
            reservation_json
        );
        return HttpResponse::InternalServerError()
            .json(ResponseError::new("Xnode could not be reserved."));
    }

    HttpResponse::Ok().json(xnode)
}

#[post("/forward_request")]
async fn forward_request(
    frequest: web::Json<networking::Request<serde_json::Value>>,
    request: HttpRequest,
) -> impl Responder {
    if !env::xnodes().contains(&frequest.xnode_id) {
        return HttpResponse::BadRequest().json(ResponseError::new("Invalid Xnode id."));
    }

    let reserved_by: String;
    if let Some(addr) = request.peer_addr() {
        reserved_by = format!("{}", addr.ip());
    } else {
        return HttpResponse::BadRequest()
            .json(ResponseError::new("IP address on connection is not set."));
    }

    let xnode = get_xnode(frequest.xnode_id.clone());
    match xnode.reservation {
        Some(reservation) => {
            if reservation.reserved_by != reserved_by {
                return HttpResponse::BadRequest().json(ResponseError::new(format!(
                    "Xnode is reserved by {}, not {}.",
                    reservation.reserved_by, reserved_by
                )));
            }
        }
        Option::None => {
            return HttpResponse::BadRequest().json(ResponseError::new("Xnode is not reserved."));
        }
    }

    if let Some(response) = as_client(&frequest.xnode_id, |client| {
        networking::request(client, &frequest).err()
    }) {
        return response;
    }

    HttpResponse::Ok().finish()
}
