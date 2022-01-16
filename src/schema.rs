table! {
    use diesel::sql_types::*;
    use crate::models::*;

    inventory (id) {
        id -> Int4,
        warehouse -> Nullable<Int4>,
        weight -> Int2,
        value -> Int2,
        transport -> PgTransport,
        dimensions -> PgDimensions,
    }
}

table! {
    use diesel::sql_types::*;

    warehouses (id) {
        id -> Int4,
        items -> Array<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(inventory, warehouses,);
