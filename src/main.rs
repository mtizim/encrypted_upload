use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer};
use content_length_middleware::ContentLengthLimit;

use crate::config::AppConfig;

mod config;
mod download;
mod upload;

mod content_length_middleware;

// TODO remove this
#[get("/")]
async fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/files" method="post" enctype="multipart/form-data">
                <input type="file" name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

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
                    .service(download::download)
                    .service(upload::upload)
                    .service(index),
            )
            .app_data(web::Data::new(config))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
