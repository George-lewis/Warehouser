use std::{fmt::Display, str::FromStr};

use diesel::{
    backend::Backend,
    pg::Pg,
    serialize::WriteTuple,
    sql_types::{SmallInt, Text},
    types::{FromSql, Record, ToSql},
    AsExpression, FromSqlRow, Insertable, Queryable,
};

use crate::schema::{inventory, warehouses};
use serde::{Deserialize, Serialize};

use diesel::result::Error as DError;

pub enum Error {
    Service(String),
    Db(diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::Db(e)
    }
}

impl Error {
    pub fn get_msg(self) -> String {
        match self {
            Error::Service(msg) => msg,
            Error::Db(err) => match err {
                DError::DatabaseError(_, info) => info.message().to_string(),
                DError::NotFound => "Item not found".to_string(),
                _ => "Unknown db error".to_string(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, FromSqlRow, AsExpression, PartialEq, Serialize, Deserialize)]
#[sql_type = "PgTransport"]
pub enum Transport {
    Air,
    Sea,
    Land,
}

impl Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(SqlType)]
#[postgres(type_name = "transport")]
pub struct PgTransport;

impl FromStr for Transport {
    type Err = ();
    fn from_str(string: &str) -> std::result::Result<Self, Self::Err> {
        let variant = match string {
            "Air" => Self::Air,
            "Sea" => Self::Sea,
            "Land" => Self::Land,
            _ => return Err(()),
        };
        Ok(variant)
    }
}

impl ToSql<PgTransport, Pg> for Transport {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        let sql = format!("{:?}", self);
        ToSql::<Text, Pg>::to_sql(&sql, out)
    }
}

impl FromSql<PgTransport, Pg> for Transport {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        let string: String = FromSql::<Text, Pg>::from_sql(bytes)?;

        // Postgresql stops us from inserting bad values
        // because we're using the enum data type
        // ergo, this should never fail
        let variant =
            Transport::from_str(&string).expect("SQL contains an invalid variant of Transport");
        Ok(variant)
    }
}

#[derive(Debug, Clone, FromSqlRow, AsExpression, PartialEq, Serialize, Deserialize)]
#[sql_type = "PgDimensions"]
pub struct Dimensions {
    pub width: i16,
    pub height: i16,
    pub depth: i16,
}

#[derive(SqlType)]
#[postgres(type_name = "dimensions")]
pub struct PgDimensions;

impl ToSql<PgDimensions, Pg> for Dimensions {
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        WriteTuple::<(SmallInt, SmallInt, SmallInt)>::write_tuple(
            &(self.width, self.height, self.depth),
            out,
        )
    }
}

impl FromSql<PgDimensions, Pg> for Dimensions {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        let (width, height, depth) =
            FromSql::<Record<(SmallInt, SmallInt, SmallInt)>, Pg>::from_sql(bytes)?;
        Ok(Self {
            width,
            height,
            depth,
        })
    }
}

#[derive(Debug, Queryable, Identifiable, AsChangeset, Insertable, Serialize, Deserialize)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "inventory"]
pub struct InventoryItem {
    pub id: i32,                // Id of this item
    pub warehouse: Option<i32>, // Optional warehouse id
    pub weight: i16,            // Weight in kg
    pub value: i16,             // Value in $
    pub transport: Transport,   // Transportation method
    pub dimensions: Dimensions, // Dimensions in m
}

#[derive(
    Debug, Clone, Queryable, Identifiable, AsChangeset, Insertable, Serialize, Deserialize,
)]
pub struct Warehouse {
    pub id: i32,         // Id of this warehouse
    pub items: Vec<i32>, // Items in the warehouse
}
