use actix_web::{middleware::Logger, web, App, HttpServer};
use endpoints::{download::download, upload::upload};
use util::content_length_middleware::ContentLengthLimit;

use crate::config::AppConfig;

mod config;
mod endpoints;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    }
    #[cfg(not(debug_assertions))]
    {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));
    }

    HttpServer::new(|| {
        let config = AppConfig::new();
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("")
                    .wrap(ContentLengthLimit::new(config.max_file_size))
                    .service(download)
                    .service(upload),
            )
            .app_data(web::Data::new(config))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
