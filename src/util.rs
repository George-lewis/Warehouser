use std::borrow::Cow;

use crate::models::{InventoryItem, Warehouse};

// Usually in Rust, you want to accept slices rather than Vecs,
// but that's not actually possible here because of the interactions with the
// generics in `request`, so we disable clippy
#[allow(clippy::ptr_arg)]
// Additionally, this function is not failable,
// but it has to conform to the return type of `request`
pub fn format_item_csv(items: &Vec<InventoryItem>) -> Result<String, String> {
    let mut csv = "id,weight,value,transport,width,height,depth\n".to_string();

    for item in items.iter() {
        let warehouse = if let Some(i) = item.warehouse {
            Cow::Owned(format!("{}", i))
        } else {
            Cow::Borrowed("null")
        };

        csv.push_str(&format!(
            "{id},{warehouse},{weight},{value},{transport},{width},{height},{depth}\n",
            id = item.id,
            warehouse = warehouse,
            weight = item.weight,
            value = item.value,
            transport = item.transport,
            width = item.dimensions.width,
            height = item.dimensions.height,
            depth = item.dimensions.depth
        ));
    }

    Ok(csv)
}

#[allow(clippy::ptr_arg)]
pub fn format_warehouse_csv(whouses: &Vec<Warehouse>) -> Result<String, String> {
    let mut csv = "id,items\n".to_string();

    for whouse in whouses.iter() {
        let id = whouse.id;

        let mut items = String::new();
        for (i, id_) in whouse.items.iter().enumerate() {
            let num = if i == 0 {
                format!("{}", id_)
            } else {
                format!(", {}", id_)
            };
            items += &num;
        }

        csv.push_str(&format!("{id},\"{items}\"\n"));
    }

    Ok(csv)
}
