use actix_web::HttpResponse;
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

pub async fn request<T: Serialize + Debug>(
    client: &Client,
    request: &Request<T>,
) -> Result<Response, HttpResponse> {
    println!("Sending request {:?}", request);

    let result = match &request.request_type {
        RequestType::Get { path } => client.get(format!("{}/{}", request.xnode_id, path)).send(),
        RequestType::Post { path, body } => client
            .post(format!("{}/{}", request.xnode_id, path))
            .json(&body)
            .send(),
    }
    .await;

    println!("Request result {:?}", result);

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            match response.text().await {
                Ok(body) => Ok(Response { status, body }),
                Err(e) => {
                    log::error!("Could not decode response {:?}: {}", request, e);
                    Err(HttpResponse::InternalServerError()
                        .json(ResponseError::new("Received invalid response.")))
                }
            }
        }
        Err(e) => {
            log::error!("Could not perform request {:?}: {}", request, e);
            Err(HttpResponse::InternalServerError()
                .json(ResponseError::new("Could not perform request.")))
        }
    }
}
