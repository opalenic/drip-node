use chrono::prelude::*;
use diesel::backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::BigInt;
use diesel::sqlite::Sqlite;
use diesel::{prelude::*, AsExpression, FromSqlRow};

use std::ops::Deref;

use crate::enviro_phat;

pub mod schema;

#[derive(Debug, Queryable)]
#[allow(dead_code)]
pub struct Measurement {
    id: i32,
    meas_time: DateTimeUtc,
    temperature: Option<f32>,
    pressure: Option<f32>,
    humidity: Option<f32>,
    light_level: Option<f32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = schema::measurements)]
pub struct InsertableMeasurement {
    meas_time: DateTimeUtc,
    temperature: Option<f32>,
    pressure: Option<f32>,
    humidity: Option<f32>,
    light_level: Option<f32>,
}

impl From<enviro_phat::Measurement> for InsertableMeasurement {
    fn from(measurement: enviro_phat::Measurement) -> Self {
        Self {
            meas_time: DateTimeUtc::now(),
            temperature: Some(measurement.temperature.0),
            pressure: Some(measurement.pressure.0),
            humidity: None,
            light_level: Some(measurement.light_level.0),
        }
    }
}

#[derive(Debug, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = BigInt)]
pub struct DateTimeUtc(DateTime<Utc>);

impl DateTimeUtc {
    pub fn now() -> DateTimeUtc {
        DateTimeUtc(Utc::now())
    }
}

impl Deref for DateTimeUtc {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromSql<BigInt, Sqlite> for DateTimeUtc
where
    i64: FromSql<BigInt, Sqlite>,
{
    fn from_sql(value: backend::RawValue<Sqlite>) -> deserialize::Result<Self> {
        let raw_val = i64::from_sql(value)?;

        Ok(DateTimeUtc(DateTime::from_utc(
            NaiveDateTime::from_timestamp(raw_val / 1_000_000, (raw_val % 1_000_000) as u32),
            Utc,
        )))
    }
}

impl ToSql<BigInt, Sqlite> for DateTimeUtc
where
    i64: ToSql<BigInt, Sqlite>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let timestamp_us =
            (self.0.timestamp() * 1_000_000) + i64::from(self.timestamp_subsec_micros());

        out.set_value(timestamp_us);

        Ok(IsNull::No)
    }
}
