use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod auth;
mod contracts;
mod enclave;
mod models;
mod utils;

use utils::{db::init_db_pool, init_metrics};

/// Main entry point for the Contract Management System
/// 
/// This function:
/// 1. Initializes the application environment and logging
/// 2. Sets up database connection and metrics
/// 3. Configures the HTTP server with middleware and routes
/// 4. Starts the server on the configured host and port
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Configure and initialize the logging system
    // This sets up structured logging with thread IDs, file names, and line numbers
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_env_filter("info")
        .init();

    info!("Starting Contract Management System...");

    // Load configuration from environment variables with fallback values
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Initialize the database connection pool
    // This creates a pool of database connections for efficient query handling
    init_db_pool(&database_url)
        .await
        .expect("Failed to initialize database pool");

    // Initialize Prometheus metrics for monitoring
    init_metrics();

    // Create authentication middleware instance with JWT secret
    // This will be used to validate JWT tokens for protected routes
    let auth = auth::AuthMiddleware::new(jwt_secret.as_bytes());

    // Configure and start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            // Add standard middleware for logging, compression, and path normalization
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            // Add authentication middleware to the application state
            .app_data(web::Data::new(auth.clone()))
            // Configure API routes under /api/v1 prefix
            .service(
                web::scope("/api/v1")
                    .configure(api::contracts_config)
                    // Additional API configurations can be added here
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run();

    info!("Server running at http://{}:{}", host, port);

    // Start the server and wait for it to complete
    server.await
}
