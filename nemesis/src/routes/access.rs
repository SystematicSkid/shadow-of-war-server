// src/routes/auth.rs
use actix_web::web;
use crate::handlers::access;

pub fn register(cfg: &mut web::ServiceConfig) {
    // Register routes at the root level (no scope)
    cfg.route("/access", web::post().to(access::get_token));
}