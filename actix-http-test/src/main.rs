use std::{thread, time::Duration};

use actix_cors::Cors;
use actix_web::{middleware::Logger, HttpResponse, Responder, get, HttpServer, App, web::Data};

mod db;
mod handlers;
mod schemas;

use self::{db::get_db_pool, handlers::register};


#[get("/")]
async fn index() -> impl Responder {
    thread::sleep(Duration::from_secs(5));
    HttpResponse::Ok().body("Hello world!")
}

#[get("/again")]
async fn again() -> impl Responder{
    HttpResponse::Ok().body("Hello again!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let pool = get_db_pool();
    
    log::info!("Starting server...");

    HttpServer::new(move ||{
        App::new()
            .app_data(Data::new(pool.clone()))
            .configure(register)
            .wrap(Logger::default())
            .service(index)
            .service(again)
            .wrap(Cors::permissive())
            .wrap(Logger::default())
    })
    .workers(2)
    .bind("localhost:8080")
    .unwrap()
    .run()
    .await
}
