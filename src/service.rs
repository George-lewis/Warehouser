use crate::models::{Error, InventoryItem, Result};
use diesel::result::Error as DError;
use diesel::PgConnection;

// Re-exports db functions
// The api layer should use service functions instead of the db module directly
// this indirection allows us to modify the internal behaviour of the functions
// in this layer, without breaking the api layer or any other dependents
pub use crate::db::*;
use crate::{db, models::Warehouse};

/// Add an item to a warehouse
pub fn warehouse_add_item(conn: &PgConnection, w_id: i32, item_id: i32) -> Result<Warehouse> {
    let mut item = db::get_item(conn, item_id).map_err(|err| {
        // Intercept the conversion to provide a more-specific error message
        if matches!(DError::NotFound, _err) {
            let msg = format!("Item id {item_id} does not exist");
            Error::Service(msg)
        } else {
            Error::from(err)
        }
    })?;

    if let Some(id_) = item.warehouse {
        let msg = if id_ == w_id {
            format!("Item id {item_id} already belongs to warehouse id {id_}")
        } else {
            format!("Cannot assign item id {item_id} to warehouse id {w_id} as it already belongs to warehouse id {id_}")
        };
        return Err(Error::Service(msg));
    }

    // Make the change
    item.warehouse = Some(w_id);

    // Update item in the db
    db::update_item(conn, &item)?;

    // We can finally modify the warehouse
    let mut whouse = db::get_warehouse(conn, w_id)?;

    if whouse.items.contains(&item_id) {
        return Err(Error::Service(format!(
            "Warehouse id {} already contains item id {}",
            w_id, item_id
        )));
    }

    whouse.items.push(item_id);
    Ok(db::update_warehouse(conn, &whouse)?)
}

pub fn warehouse_remove_item(conn: &PgConnection, w_id: i32, item_id: i32) -> Result<Warehouse> {
    let mut item = db::get_item(conn, item_id).map_err(|err| {
        // Intercept the conversion to provide a more-specific error message
        if matches!(DError::NotFound, _err) {
            let msg = format!("Item id {item_id} does not exist");
            Error::Service(msg)
        } else {
            Error::from(err)
        }
    })?;

    if let Some(id_) = item.warehouse {
        if id_ != w_id {
            let msg = format!("Item id {item_id} does not belong to warehouse id {w_id}, belongs to warehouse id {id_}");
            return Err(Error::Service(msg));
        }
    } else {
        let msg = format!("Item id {item_id} does not belong to any warehouse");
        return Err(Error::Service(msg));
    }

    // Set null warehouse
    item.warehouse = None;
    db::update_item(conn, &item)?;

    let mut whouse = db::get_warehouse(conn, w_id).map_err(|err| {
        // Intercept the conversion to provide a more-specific error message
        if matches!(DError::NotFound, _err) {
            let msg = format!("Warehouse id {w_id} does not exist");
            Error::Service(msg)
        } else {
            Error::from(err)
        }
    })?;

    let idx = whouse.items.iter().position(|&id_| id_ == item_id);
    if let Some(idx) = idx {
        whouse.items.remove(idx);
        Ok(db::update_warehouse(conn, &whouse)?)
    } else {
        // This happens if `whouse.items` does not contain `item_id`
        // which would be an inconsistent state
        let msg = format!("INCONSISTENCY IN DATABASE: Item id {item_id} claims it belongs to warehouse id {w_id}, 
            however warehouse id {w_id} does not indicate ownership");
        Err(Error::Service(msg))
    }
}

// Even though we re-export db::delete_item
// we're making a custom implementation here
/// Delete an item
pub fn delete_item(conn: &PgConnection, item_id: i32) -> Result<InventoryItem> {
    let item = db::get_item(conn, item_id)?;

    if let Some(w_id) = item.warehouse {
        // Stubbed
        warehouse_remove_item(conn, w_id, item_id).ok();
    }

    Ok(db::delete_item(conn, item_id)?)
}

pub fn create_item(conn: &PgConnection, item: &InventoryItem) -> Result<InventoryItem> {
    if let Some(w_id) = item.warehouse {
        if let Err(DError::NotFound) = db::get_warehouse(conn, w_id) {
            let msg =
                format!("Cannot create item with warehouse id {w_id}, because it does not exist");
            return Err(Error::Service(msg));
        }
    }

    Ok(db::insert_item(conn, item)?)
}

pub fn create_warehouse(conn: &PgConnection, whouse: &Warehouse) -> Result<Warehouse> {
    let check = db::get_warehouse(conn, whouse.id);
    if check.is_ok() {
        let msg = format!("Warehouse id {} already exists", whouse.id);
        return Err(Error::Service(msg));
    }

    // We need to check a few things
    // Potential concurrency issue?: What if the items are changed between here and adding them to the warehouse?
    for &item_id in &whouse.items {
        let item = db::get_item(conn, item_id);
        let item = if let Err(DError::NotFound) = item {
            let msg = format!("Cannot create warehouse, item id {item_id} does not exist");
            return Err(Error::Service(msg));
        } else {
            item
        }?;

        if let Some(w_id) = item.warehouse {
            let msg = format!(
                "Cannot create warehouse, item id {item_id} already belongs to warehouse id {w_id}"
            );
            return Err(Error::Service(msg));
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
