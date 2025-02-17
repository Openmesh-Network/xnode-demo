use std::fs::create_dir_all;

use actix_web::{web, App, HttpServer};
use utils::env::{datadir, hostname, port, reservationsdir};

mod demo;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create data directories
    {
        let dir = datadir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!("Could not create data dir at {}: {}", dir.display(), e)
        })?;
    }
    {
        let dir = reservationsdir();
        create_dir_all(&dir).inspect_err(|e| {
            log::error!(
                "Could not create reservations dir at {}: {}",
                dir.display(),
                e
            )
        })?;
    }

    // Start server
    HttpServer::new(move || App::new().service(web::scope("/demo").configure(demo::configure)))
        .bind(format!("{}:{}", hostname(), port()))?
        .run()
        .await
}
