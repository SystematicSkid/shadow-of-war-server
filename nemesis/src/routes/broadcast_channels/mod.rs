use actix_web::web;
use crate::handlers::broadcast_channels::motd;
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/broadcast_channels")
        .service(motd::get));
}