use actix_web::web::ServiceConfig;

pub mod handlers;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(handlers::xnodes).service(handlers::address);
    cfg.service(handlers::xnodes).service(handlers::xnodes);
    cfg.service(handlers::xnodes).service(handlers::reserve);
    cfg.service(handlers::xnodes)
        .service(handlers::forward_request);
}
