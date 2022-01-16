//! Warehouser
//! Author: George Lewis (george@georgelewis.ca)
//! Copyright reserved
//! For Shopify's Backend Challenge Summer 2022

#[macro_use]
extern crate diesel;
extern crate serde;

pub mod api;
pub mod db;
pub mod models;
pub mod schema;
pub mod service;
pub mod util;

use actix_web::{error::InternalError, middleware::Logger, web, App, HttpResponse, HttpServer};

use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use dotenv::dotenv;
use r2d2::Pool;

use api::*;

type DbPool = Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Warehouser Startup!");

    dotenv().expect("Couldn't load .env");
    std::env::set_var("RUST_LOG", "actix_web=info");

    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::builder()
        .max_size(1) // Mitigation for buggy behaviour with Postgresql 14
        .build(manager)
        .expect("Couldn't create db pool");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(pool.clone())
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                let fmt = format!("{}", &err);
                let resp = HttpResponse::InternalServerError().body(fmt);
                InternalError::from_response(err, resp).into()
            }))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/item")
                            .service(create_item) // C
                            .service(get_items) // R
                            .service(update_item) // U
                            .service(delete_item) // D
                            .service(item_csv)
                            .service(get_item),
                    )
                    .service(
                        web::scope("/warehouse")
                            .service(warehouse_csv)
                            .service(warehouse_add_item)
                            .service(warehouse_remove_item)
                            .service(warehouse_get_items)
                            .service(create_warehouse)
                            .service(get_warehouse)
                            .service(get_warehouses)
                            .service(delete_warehouse)
                            .service(update_warehouse)
                    ),
            )
    })
    .bind("127.0.0.1:8087")
    .expect("Couldn't bind")
    .run()
    .await
}
