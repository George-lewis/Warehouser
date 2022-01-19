use crate::models::{Error, InventoryItem, Result};
use actix_web::http::StatusCode;
use diesel::PgConnection;

// Re-exports db functions
// The api layer should use service functions instead of the db module directly
// this indirection allows us to modify the internal behaviour of the functions
// in this layer, without breaking the api layer or any other dependents
pub use crate::db::*;
use crate::{db, models::NotFound, models::Warehouse};

/// Add an item to a warehouse
pub fn warehouse_add_item(conn: &PgConnection, w_id: i32, item_id: i32) -> Result<Warehouse> {
    let mut item =
        db::get_item(conn, item_id).not_found(|| format!("Item id {item_id} does not exist"))?;

    if let Some(id_) = item.warehouse {
        let msg = if id_ == w_id {
            format!("Item id {item_id} already belongs to warehouse id {id_}")
        } else {
            format!("Cannot assign item id {item_id} to warehouse id {w_id} as it already belongs to warehouse id {id_}")
        };
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        });
    }

    // Make the change
    item.warehouse = Some(w_id);

    // Update item in the db
    db::update_item(conn, &item)?;

    // We can finally modify the warehouse
    let mut whouse = db::get_warehouse(conn, w_id)?;

    if whouse.items.contains(&item_id) {
        return Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("INCONSISTENCY IN DATABASE: Item id {item_id} claims it belongs to no warehouse, yet warehouse id {w_id} indicates ownership"),
        });
    }

    whouse.items.push(item_id);
    db::update_warehouse(conn, &whouse)
}

pub fn warehouse_remove_item(conn: &PgConnection, w_id: i32, item_id: i32) -> Result<Warehouse> {
    let mut item =
        db::get_item(conn, item_id).not_found(|| format!("Item id {item_id} does not exist"))?;

    if let Some(id_) = item.warehouse {
        if id_ != w_id {
            let msg = format!("Item id {item_id} does not belong to warehouse id {w_id}, belongs to warehouse id {id_}");
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg,
            });
        }
    } else {
        let msg = format!("Item id {item_id} does not belong to any warehouse");
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        });
    }

    // Set null warehouse
    item.warehouse = None;
    db::update_item(conn, &item)?;

    let mut whouse = db::get_warehouse(conn, w_id)
        .not_found(|| format!("Warehouse id {w_id} does not exist"))?;

    let idx = whouse.items.iter().position(|&id_| id_ == item_id);
    if let Some(idx) = idx {
        whouse.items.remove(idx);
        Ok(db::update_warehouse(conn, &whouse)?)
    } else {
        // This happens if `whouse.items` does not contain `item_id`
        // which would be an inconsistent state
        let msg = format!("INCONSISTENCY IN DATABASE: Item id {item_id} claims it belongs to warehouse id {w_id}, 
            however warehouse id {w_id} does not indicate ownership");
        Err(Error {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg,
        })
    }
}

// Even though we re-export db::delete_item
// we're making a custom implementation here
/// Delete an item
pub fn delete_item(conn: &PgConnection, item_id: i32) -> Result<InventoryItem> {
    let item = db::get_item(conn, item_id)?;

    if let Some(w_id) = item.warehouse {
        warehouse_remove_item(conn, w_id, item_id)?;
    }

    // If we returned the result of this
    // we would be potentially be incorrectly showing
    // the item as being in no warehouse
    db::delete_item(conn, item_id)?;

    Ok(item)
}

pub fn create_item(conn: &PgConnection, item: &InventoryItem) -> Result<InventoryItem> {
    if let Some(w_id) = item.warehouse {
        // Check for warehouse existence
        db::get_warehouse(conn, w_id).not_found(|| {
            format!("Cannot create item with warehouse id {w_id}, because it does not exist")
        })?;
    }

    db::insert_item(conn, item)
}

pub fn create_warehouse(conn: &PgConnection, whouse: &Warehouse) -> Result<Warehouse> {
    if db::get_warehouse(conn, whouse.id).is_ok() {
        let msg = format!("Warehouse id {} already exists", whouse.id);
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        });
    }

    // We need to check a few things
    // Potential concurrency issue?: What if the items are changed between here and adding them to the warehouse?
    for &item_id in &whouse.items {
        let item = db::get_item(conn, item_id)
            .not_found(|| format!("Cannot create warehouse, item id {item_id} does not exist"))?;

        if let Some(w_id) = item.warehouse {
            let msg = format!(
                "Cannot create warehouse, item id {item_id} already belongs to warehouse id {w_id}"
            );
            return Err(Error {
                code: StatusCode::BAD_REQUEST,
                msg,
            });
        }
    }

    // A little cheat.
    // `warehouse_add_item` will fail if the warehouse says
    // that it already contains the item
    // so we make a copy of the warehouse with an empty
    // `items` and add them after
    let mut cloned = whouse.clone();
    cloned.items.clear();

    let mut created = db::insert_warehouse(conn, &cloned)?;

    for &item_id in &whouse.items {
        warehouse_add_item(conn, created.id, item_id)?;
    }

    // We could do another fetch to the database
    // but if the add calls didn't fail this is a fairly safe bet
    // tradeoff!
    created.items.extend_from_slice(&whouse.items);

    Ok(created)
}

pub fn delete_warehouse(conn: &PgConnection, w_id: i32) -> Result<Warehouse> {
    let whouse = db::get_warehouse(conn, w_id)?;

    for &item_id in &whouse.items {
        warehouse_remove_item(conn, w_id, item_id)?;
    }

    db::delete_warehouse(conn, w_id)?;

    // If we did `Ok(deleted)` it wouldn't show the items
    Ok(whouse)
}

pub fn update_warehouse(_: &PgConnection, _: &Warehouse) -> Result<Warehouse> {
    // Technically a warehouse can be updated in the db, db::update_warehouse does exist
    // but it doesn't make sense to update the id, and we're not supporting updating
    // the list of items, for that use the add and remove item endpoints

    let err = Error {
        code: StatusCode::NOT_IMPLEMENTED,
        msg: "Updating warehouses is not supported, to add and remove items use the respective endpoints".to_string()
    };
    Err(err)
}

pub fn update_item(conn: &PgConnection, item: &InventoryItem) -> Result<InventoryItem> {
    let db_item = db::get_item(conn, item.id).not_found(|| {
        format!(
            "Cannot update item {} as it doesn't exist. Try creating the item instead",
            item.id
        )
    })?;

    // We want to enforce that you can't update an item's warehouse via this endpoint
    if item.warehouse != db_item.warehouse {
        let msg = "Updating an item's warehouse is not supported, use the warehouse item add/remove endpoint".to_string();
        return Err(Error {
            code: StatusCode::BAD_REQUEST,
            msg,
        });
    }

    // Forward
    db::update_item(conn, item)
}

pub fn warehouse_get_items(
    conn: &PgConnection,
    w_id: i32,
    limit: i64,
) -> Result<Vec<InventoryItem>> {
    let whouse = db::get_warehouse(conn, w_id)
        .not_found(|| format!("Cannot get items for warehouse id {w_id}, as it does not exist"))?;

    db::get_items_by_id(conn, limit, &whouse.items)
}
