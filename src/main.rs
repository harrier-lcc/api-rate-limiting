use actix_web::{App, HttpServer};
use api_rate_limiting::app_config::config_app;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    // Simple logger just for the ease of development
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| App::new().configure(config_app))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
