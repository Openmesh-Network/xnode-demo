use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::user);
    cfg.service(handlers::xnodes);
    cfg.service(handlers::reserve);
}
