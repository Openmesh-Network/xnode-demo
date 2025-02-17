use actix_web::HttpResponse;
use futures::executor::block_on;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::utils::error::ResponseError;

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestType<T: Serialize> {
    Get { path: String },
    Post { path: String, body: T },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<T: Serialize> {
    pub xnode_id: String,
    pub request_type: RequestType<T>,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

pub fn request<T: Serialize + Debug>(
    client: &Client,
    request: &Request<T>,
) -> Result<Response, HttpResponse> {
    let result = match &request.request_type {
        RequestType::Get { path } => {
            block_on(client.get(format!("{}/{}", request.xnode_id, path)).send())
        }
        RequestType::Post { path, body } => block_on(
            client
                .post(format!("{}/{}", request.xnode_id, path))
                .json(&body)
                .send(),
        ),
    };

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            match block_on(response.text()) {
                Ok(body) => Ok(Response { status, body }),
                Err(e) => {
                    log::error!("Could not decode response {:?}: {}", request, e);
                    return Err(HttpResponse::InternalServerError()
                        .json(ResponseError::new("Received invalid response.")));
                }
            }
        }
        Err(e) => {
            log::error!("Could not perform request {:?}: {}", request, e);
            return Err(HttpResponse::InternalServerError()
                .json(ResponseError::new("Could not perform request.")));
        }
    }
}
