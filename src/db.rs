use crate::diesel::ExpressionMethods;
/// Wraps common database operations
/// These functions should not be used directly
/// Instead, use the functions `crate::service`
// Implementation note:
// This code is fairly repetitive
// it could fairly easily be reduced by creating a macro
// or something else?
use diesel::dsl::any;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};

use crate::models::{InventoryItem, Result, Warehouse};

pub fn get_items_by_id(conn: &PgConnection, limit: i64, ids: &[i32]) -> Result<Vec<InventoryItem>> {
    use crate::schema::inventory::dsl::*;

    inventory
        .limit(limit)
        .filter(id.eq(any(ids)))
        .get_results(conn)
        .map_err(Into::into)
}

pub fn get_items(conn: &PgConnection, limit: i64) -> Result<Vec<InventoryItem>> {
    use crate::schema::inventory::dsl::*;

    inventory.limit(limit).get_results(conn).map_err(Into::into)
}

pub fn get_item(conn: &PgConnection, id_: i32) -> Result<InventoryItem> {
    use crate::schema::inventory::dsl::*;

    inventory.find(id_).first(conn).map_err(Into::into)
}

pub fn insert_item(conn: &PgConnection, item: &InventoryItem) -> Result<InventoryItem> {
    use crate::schema::inventory::dsl::*;

    diesel::insert_into(inventory)
        .values(item)
        .get_result(conn)
        .map_err(Into::into)
}

pub fn update_item(conn: &PgConnection, item: &InventoryItem) -> Result<InventoryItem> {
    use crate::schema::inventory::dsl::*;

    diesel::update(inventory)
        .filter(id.eq(item.id))
        .set(item)
        .get_result(conn)
        .map_err(Into::into)
}

pub fn delete_item(conn: &PgConnection, id_: i32) -> Result<InventoryItem> {
    use crate::schema::inventory::dsl::*;

    diesel::delete(inventory)
        .filter(id.eq(id_))
        .get_result(conn)
        .map_err(Into::into)
}

pub fn get_warehouses_by_id(
    conn: &PgConnection,
    limit: i64,
    ids: &[i32],
) -> Result<Vec<Warehouse>> {
    use crate::schema::warehouses::dsl::*;

    warehouses
        .limit(limit)
        .filter(id.eq(any(ids)))
        .get_results(conn)
        .map_err(Into::into)
}

pub fn get_warehouses(conn: &PgConnection, limit: i64) -> Result<Vec<Warehouse>> {
    use crate::schema::warehouses::dsl::*;

    warehouses
        .limit(limit)
        .get_results(conn)
        .map_err(Into::into)
}

pub fn get_warehouse(conn: &PgConnection, id_: i32) -> Result<Warehouse> {
    use crate::schema::warehouses::dsl::*;

    warehouses.find(id_).first(conn).map_err(Into::into)
}

pub fn insert_warehouse(conn: &PgConnection, whouse: &Warehouse) -> Result<Warehouse> {
    use crate::schema::warehouses::dsl::*;

    diesel::insert_into(warehouses)
        .values(whouse)
        .get_result(conn)
        .map_err(Into::into)
}

pub fn update_warehouse(conn: &PgConnection, whouse: &Warehouse) -> Result<Warehouse> {
    use crate::schema::warehouses::dsl::*;

    diesel::update(warehouses)
        .filter(id.eq(whouse.id))
        .set(whouse)
        .get_result(conn)
        .map_err(Into::into)
}

pub fn delete_warehouse(conn: &PgConnection, id_: i32) -> Result<Warehouse> {
    use crate::schema::warehouses::dsl::*;

    diesel::delete(warehouses)
        .filter(id.eq(id_))
        .get_result(conn)
        .map_err(Into::into)
}
