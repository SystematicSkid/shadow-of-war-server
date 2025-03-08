// src/routes/auth.rs
use actix_web::web;
use crate::handlers::auth;

pub fn register(cfg: &mut web::ServiceConfig) {
    // Register routes at the root level (no scope)
    cfg.route("/auth", web::post().to(auth::authenticate));
}