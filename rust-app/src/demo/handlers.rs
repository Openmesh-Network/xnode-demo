use actix_web::{body::BoxBody, get, http::StatusCode, post, web, HttpResponse, Responder};
use hex::ToHex;
use rand::{rng, Rng};
use std::fs::write;
use std::time::SystemTime;

use crate::{
    demo::models::{ForwardRequest, PublicXnode, Reserve, SetApp},
    utils::{
        auth::as_client,
        env::{self, reservationduration, reservationsdir},
        error::ResponseError,
        networking,
        wallet::get_signer,
        xnode::{
            get_xnode, get_xnodes, path_safe_id, ConfigurationAction, ContainerConfiguration,
            Reservation,
        },
    },
};

#[get("/address")]
async fn address() -> impl Responder {
    let signer = get_signer();
    let address: String = signer.public().address().encode_hex();

    HttpResponse::Ok().json(address)
}

#[get("/xnodes")]
async fn xnodes() -> impl Responder {
    let xnodes: Vec<PublicXnode> = get_xnodes()
        .await
        .into_iter()
        .map(|xnode| PublicXnode {
            id: xnode.id,
            reserved_until: xnode
                .reservation
                .map(|reservation| reservation.reserved_until),
        })
        .collect();

    HttpResponse::Ok().json(xnodes)
}

#[post("/reserve")]
async fn reserve(reserve: web::Json<Reserve>) -> impl Responder {
    let xnode_id = reserve.xnode_id.clone();
    if !env::xnodes().contains(&xnode_id) {
        return HttpResponse::BadRequest().json(ResponseError::new("Invalid Xnode id."));
    }

    let xnode = get_xnode(xnode_id).await;
    if xnode.reservation.is_some() {
        return HttpResponse::BadRequest().json(ResponseError::new("Xnode is already reserved."));
    }

    let secret = generate_secret();

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
        secret,
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

    let path = reservationsdir().join(path_safe_id(&xnode.id));
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

    HttpResponse::Ok().json(get_xnode(xnode.id).await)
}

#[post("/set_app")]
async fn set_app(app: web::Json<SetApp>) -> impl Responder {
    if let Some(response) = check_reservation(&app.xnode_id, &app.secret).await {
        return response;
    }

    match as_client(&app.xnode_id.clone(), |client| async move {
        networking::request(
            &client,
            &networking::Request {
                xnode_id: app.xnode_id.clone(),
                request_type: networking::RequestType::Post {
                    path: String::from("config/change"),
                    body: vec![ConfigurationAction::Set {
                        container: app.secret.clone(),
                        config: ContainerConfiguration {
                            flake: app.flake.clone(),
                        },
                    }],
                },
            },
        )
        .await
    })
    .await
    {
        Ok(forward_response) => respond(forward_response),
        Err(e) => e,
    }
}

#[post("/forward_request")]
async fn forward_request(frequest: web::Json<ForwardRequest>) -> impl Responder {
    {
        let (networking::RequestType::Get { path }
        | networking::RequestType::Post { path, body: _ }) = &frequest.request.request_type;
        if path.starts_with("processes") {
            // Reserver only endpoint
            if let Some(response) =
                check_reservation(&frequest.request.xnode_id, &frequest.secret).await
            {
                return response;
            }
        } else if !path.starts_with("usage") {
            // Not a public endpoint
            return HttpResponse::BadRequest().json(ResponseError::new("Invalid path."));
        }
    }

    match as_client(&frequest.request.xnode_id.clone(), |client| async move {
        networking::request(&client, &frequest.request).await
    })
    .await
    {
        Ok(forward_response) => respond(forward_response),
        Err(e) => e,
    }
}

async fn check_reservation(xnode_id: &String, secret: &String) -> Option<HttpResponse> {
    if !env::xnodes().contains(xnode_id) {
        return Some(HttpResponse::BadRequest().json(ResponseError::new("Invalid Xnode id.")));
    }

    let xnode = get_xnode(xnode_id.clone()).await;
    match xnode.reservation {
        Some(reservation) => {
            if &reservation.secret != secret {
                return Some(
                    HttpResponse::BadRequest().json(ResponseError::new(String::from(
                        "Invalid reservation secret.",
                    ))),
                );
            }
        }
        Option::None => {
            return Some(
                HttpResponse::BadRequest().json(ResponseError::new("Xnode is not reserved.")),
            );
        }
    }

    None
}

fn respond(response: networking::Response) -> HttpResponse {
    match StatusCode::from_u16(response.status) {
        Ok(status_code) => HttpResponse::with_body(status_code, BoxBody::new(response.body)),
        Err(e) => {
            log::error!(
                "Could not translate reqwest status code {} into actix status code: {}",
                response.status,
                e
            );
            HttpResponse::InternalServerError().json(ResponseError::new(
                "Status code of response could not be parsed.",
            ))
        }
    }
}

fn generate_secret() -> String {
    let secret: String = rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    format!("demoprefix{}", secret)
}
