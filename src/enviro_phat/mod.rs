use anyhow::Result;
use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::db::{self, InsertableMeasurement};

use std::sync::{Arc, Mutex};

#[cfg(feature = "enviro-phat-v1")]
mod v1;
#[cfg(feature = "enviro-phat-v1")]
pub use v1::EnviroPHatV1 as EnviroPHat;

#[cfg(feature = "enviro-phat-stub")]
mod stub;
#[cfg(feature = "enviro-phat-stub")]
pub use stub::EnviroPHatStub as EnviroPHat;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Temperature(pub f32);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Pressure(pub f32);
#[derive(Debug, PartialEq, PartialOrd)]
pub struct LightLevel(pub f32);

#[derive(Debug)]
pub struct Measurement {
    pub pressure: Pressure,
    pub temperature: Temperature,
    pub light_level: LightLevel,
}

pub trait MeasureEnvironment {
    fn measure(&self) -> Result<Measurement>;
}

pub fn create_measurement_task(
    enviro_phat: Arc<EnviroPHat>,
    db_conn: Arc<Mutex<SqliteConnection>>,
) -> impl FnOnce() -> Result<()> {
    move || {
        log::trace!("Performing measurement on {enviro_phat:?}.");
        let measurement = enviro_phat.measure()?;
        log::trace!("Measured values: {measurement:?}");

        Ok({
            use db::schema::measurements::dsl::*;
            let mut locked_db_conn = db_conn.lock().unwrap();

            log::trace!("Inserting measurement into DB.");
            let insertable = InsertableMeasurement::from(measurement);
            diesel::insert_into(measurements)
                .values(&insertable)
                .execute(&mut *locked_db_conn)?;
        })
    }
}
