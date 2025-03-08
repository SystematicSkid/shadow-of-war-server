pub mod auth;
pub mod access;
pub mod ssc;
use actix_web::web;

/// Registers all routes with the application
pub fn register(cfg: &mut web::ServiceConfig) {
    auth::register(cfg);
    access::register(cfg);
    ssc::register(cfg);
}