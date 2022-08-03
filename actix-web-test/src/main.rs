use actix_web::{middleware::Logger, HttpResponse, Responder, get, HttpServer, App};


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/again")]
async fn again() -> impl Responder{
    HttpResponse::Ok().body("Hello again!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Starting server...");

    HttpServer::new(||{
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(again)
    })
    .bind("localhost:8080")
    .unwrap()
    .run()
    .await
}
