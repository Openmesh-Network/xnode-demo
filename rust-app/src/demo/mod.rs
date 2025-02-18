use actix_web::web::ServiceConfig;

pub mod handlers;
pub mod models;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::xnodes).service(handlers::address);
    cfg.service(handlers::xnodes).service(handlers::xnodes);
    cfg.service(handlers::xnodes).service(handlers::reserve);
    cfg.service(handlers::xnodes).service(handlers::set_app);
    cfg.service(handlers::xnodes)
        .service(handlers::forward_request);
}
