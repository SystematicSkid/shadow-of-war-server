use nemesis::config::Config;
use nemesis::server;
use std::process;
use tracing::{error, info};
use tracing_subscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if present
    dotenv::dotenv().ok();
    
    // Initialize configuration
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };
    
    // Initialize logger
    if let Err(e) = server::init_logger(&config) {
        error!("Failed to initialize logger: {}", e);
        process::exit(1);
    }
    
    info!("Starting Nemesis backend server");
    
    // Start the server
    server::run(config).await
}