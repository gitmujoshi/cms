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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
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

    // Get configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    // Initialize database connection
    init_db_pool(&database_url)
        .await
        .expect("Failed to initialize database pool");

    // Initialize metrics
    init_metrics();

    // Initialize authentication middleware
    let auth = auth::AuthMiddleware::new(jwt_secret.as_bytes());

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            // Add middleware
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            // Add state
            .app_data(web::Data::new(auth.clone()))
            // Add routes
            .service(
                web::scope("/api/v1")
                    .configure(api::contracts_config)
                    // Add more API configurations here
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run();

    info!("Server running at http://{}:{}", host, port);

    server.await
}
