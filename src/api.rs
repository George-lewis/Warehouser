/// The api layer
/// This file contains all of the endpoints of the server
// There's a little bit of duplicated code going around for each endpoint
// it could in theory be shortned once again with macros, but at the cost of flexibility
use actix_web::{
    delete, error::BlockingError, get, http::StatusCode, post, put, web, HttpResponse, Responder,
};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    models::{self, Error, InventoryItem, Warehouse},
    service,
    util::{format_item_csv, format_warehouse_csv},
    DbPool,
};

/// Default limit for batch queries
const DEFAULT_LIMIT: i64 = 100;

// Payloads

#[derive(Deserialize)]
pub struct IdPayload {
    id: i32,
}

#[derive(Deserialize)]
pub struct LimitPayload {
    limit: Option<i64>,
}

impl LimitPayload {
    // Convenience method
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT)
    }
}

/// Implements a lot of default behaviour for api endpoints
///
/// ## Parameters
/// * `pool` - The `DbPool` for this request
/// * `ser` - A function ptr that serializes the `Output` of your requester
/// * `req` - A function that produces a serializable result,
/// usually a database action. This function is executed in a blocking context.
/// * `status` - The status to return on success

// A note on efficiency
// The Rust compiler is *pretty smart*
// this function will be monomorphized for each instance
// of its generic parameters. The compiler may
// inline a few things, like the function pointers
// and closures, and may even inline this function
// into the calling location.
pub async fn request<ErrorType, Output, Requester>(
    pool: web::Data<DbPool>,
    ser: fn(&Output) -> Result<String, ErrorType>,
    req: Requester,
    status: StatusCode,
) -> impl Responder
where
    ErrorType: Debug,
    Output: 'static + Serialize + Send,
    Requester: 'static + Fn(&PgConnection) -> models::Result<Output> + Send,
{
    // When Diesel is updated to support async, this can be moved out
    let future: Result<Output, BlockingError<Error>> = web::block(move || {
        // Get a db handle from the connection pool
        let conn = pool.get().map_err(|_| Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: "Couldn't get a db connection".to_owned(),
        })?;

        // Execute the user request
        req(&conn)
    })
    .await;

    match future {
        Ok(out) => {
            // Success, serialize the body
            let formatted = ser(&out).expect("Failed to serialize!");
            HttpResponse::build(status).body(formatted)
        }
        Err(BlockingError::Error(e)) => HttpResponse::build(e.code).body(e.msg),
        Err(BlockingError::Canceled) => HttpResponse::InternalServerError()
            .body("Unexpected error: Blocking operating cancelled"),
    }
}

#[post("")]
pub async fn create_item(
    pool: web::Data<DbPool>,
    query: web::Json<InventoryItem>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::create_item(conn, &query),
        StatusCode::CREATED,
    )
    .await
}

#[get("/{id}")]
pub async fn get_item(pool: web::Data<DbPool>, path: web::Path<IdPayload>) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::get_item(conn, path.id),
        StatusCode::OK,
    )
    .await
}

#[get("")]
pub async fn get_items(pool: web::Data<DbPool>, query: web::Query<LimitPayload>) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::get_items(conn, query.limit()),
        StatusCode::OK,
    )
    .await
}

#[put("")]
pub async fn update_item(
    pool: web::Data<DbPool>,
    data: web::Json<InventoryItem>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::update_item(conn, &data),
        StatusCode::OK,
    )
    .await
}

#[delete("/{id}")]
pub async fn delete_item(pool: web::Data<DbPool>, path: web::Path<IdPayload>) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::delete_item(conn, path.id),
        StatusCode::OK,
    )
    .await
}

#[get("/csv")]
pub async fn item_csv(pool: web::Data<DbPool>, query: web::Query<LimitPayload>) -> impl Responder {
    request(
        pool,
        format_item_csv,
        move |conn| service::get_items(conn, query.limit()),
        StatusCode::OK,
    )
    .await
}

#[get("/{id}/items")]
pub async fn warehouse_get_items(
    pool: web::Data<DbPool>,
    path: web::Path<IdPayload>,
    query: web::Query<LimitPayload>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::warehouse_get_items(conn, path.id, query.limit()),
        StatusCode::OK,
    )
    .await
}

#[post("/{id}/add")]
pub async fn warehouse_add_item(
    pool: web::Data<DbPool>,
    path: web::Path<IdPayload>,
    query: web::Query<IdPayload>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::warehouse_add_item(conn, path.id, query.id),
        StatusCode::OK,
    )
    .await
}

#[post("/{id}/remove")]
pub async fn warehouse_remove_item(
    pool: web::Data<DbPool>,
    path: web::Path<IdPayload>,
    query: web::Query<IdPayload>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::warehouse_remove_item(conn, path.id, query.id),
        StatusCode::OK,
    )
    .await
}

#[post("")]
pub async fn create_warehouse(
    pool: web::Data<DbPool>,
    data: web::Json<Warehouse>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::create_warehouse(conn, &data),
        StatusCode::CREATED,
    )
    .await
}

#[get("")]
pub async fn get_warehouses(
    pool: web::Data<DbPool>,
    query: web::Query<LimitPayload>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::get_warehouses(conn, query.limit()),
        StatusCode::OK,
    )
    .await
}

#[get("/{id}")]
pub async fn get_warehouse(pool: web::Data<DbPool>, path: web::Path<IdPayload>) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::get_warehouse(conn, path.id),
        StatusCode::OK,
    )
    .await
}

#[get("/csv")]
pub async fn warehouse_csv(
    pool: web::Data<DbPool>,
    query: web::Query<LimitPayload>,
) -> impl Responder {
    request(
        pool,
        format_warehouse_csv,
        move |conn| service::get_warehouses(conn, query.limit()),
        StatusCode::OK,
    )
    .await
}

#[delete("/{id}")]
pub async fn delete_warehouse(
    pool: web::Data<DbPool>,
    path: web::Path<IdPayload>,
) -> impl Responder {
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::delete_warehouse(conn, path.id),
        StatusCode::OK,
    )
    .await
}

#[put("")]
pub async fn update_warehouse(
    pool: web::Data<DbPool>,
    data: web::Json<Warehouse>,
) -> impl Responder {
    println!("hi?");
    request(
        pool,
        serde_json::to_string_pretty,
        move |conn| service::update_warehouse(conn, &data),
        StatusCode::OK,
    )
    .await
}
