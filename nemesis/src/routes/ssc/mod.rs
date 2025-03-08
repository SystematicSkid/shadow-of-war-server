use actix_web::web;
use crate::handlers::ssc::get_server_time;
/// Registers all routes with the ssc/invoke 
pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ssc/invoke")
        .service(get_server_time::get));
}