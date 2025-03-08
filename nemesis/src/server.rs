use crate::config::Config;
use crate::routes;
use actix_web::{App, HttpServer, web, middleware};
use crate::handlers::default::not_found_handler;
use anyhow::Result;
use std::io;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use crate::middleware::logger::RequestLogger;
use actix_web::middleware::Logger;

pub fn init_logger(config: &Config) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));
    
    let registry = tracing_subscriber::registry().with(env_filter);
    
    match config.logging.format.as_str() {
        "json" => {
            if let Some(file_path) = &config.logging.file_path {
                let file_appender = RollingFileAppender::new(
                    Rotation::DAILY,
                    file_path,
                    "nemesis-backend.log",
                );
                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                let json_layer = fmt::Layer::default()
                    .json()
                    .with_writer(non_blocking);
                
                registry.with(json_layer).init();
            } else {
                let json_layer = fmt::Layer::default().json();
                registry.with(json_layer).init();
            }
        }
        _ => {
            if let Some(file_path) = &config.logging.file_path {
                let file_appender = RollingFileAppender::new(
                    Rotation::DAILY,
                    file_path,
                    "nemesis-backend.log",
                );
                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
                let file_layer = fmt::Layer::default().with_writer(non_blocking);
                let console_layer = fmt::Layer::default();
                
                registry.with(file_layer).with(console_layer).init();
            } else {
                let console_layer = fmt::Layer::default();
                registry.with(console_layer).init();
            }
        }
    }
    
    Ok(())
}

pub async fn run(config: Config) -> io::Result<()> {
    let server_addr = format!("{}:{}", config.server.host, config.server.port);
    let workers = config.server.workers;
    
    info!("Starting server at http://{}", server_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(RequestLogger::new())
            .wrap(middleware::Compress::default())
            .configure(routes::register)
            .default_service(web::route().to(not_found_handler))
    })
    .workers(workers)
    .bind(&server_addr)?
    .run()
    .await
}