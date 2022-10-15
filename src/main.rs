use anyhow::Result;
use lazy_static::lazy_static;
use tokio::{select, task, time};

use std::{path::PathBuf, sync::Arc, time::Duration};

mod enviro_phat;
use enviro_phat::{EnviroPHat, MeasureEnvironment};

use diesel::prelude::*;

mod db;
use db::{InsertableMeasurement, Measurement};

lazy_static! {
    static ref CONFIG: GlobalConfig = GlobalConfig::from_env().unwrap();
}

#[derive(Debug)]
struct GlobalConfig {
    i2c_bus_path: PathBuf,
    measurement_period: Duration,
    db_path: PathBuf,
}

impl GlobalConfig {
    const I2C_DEV_PATH_ENV_VAR: &'static str = "I2C_DEV_PATH";
    const MEASUREMENT_PERIOD_ENV_VAR: &'static str = "MEASUREMENT_PERIOD_SECS";
    const DB_FILE_PATH_ENV_VAR: &'static str = "DATABASE_URL";

    fn from_env() -> Result<Self> {
        dotenv::dotenv()?;
        let i2c_bus_path = PathBuf::from(dotenv::var(Self::I2C_DEV_PATH_ENV_VAR)?);

        let measurement_period_secs = dotenv::var(Self::MEASUREMENT_PERIOD_ENV_VAR)?.parse()?;
        let measurement_period = Duration::from_secs(measurement_period_secs);

        let db_path = PathBuf::from(dotenv::var(Self::DB_FILE_PATH_ENV_VAR)?);

        Ok(Self {
            i2c_bus_path,
            measurement_period,
            db_path,
        })
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Hello, world!");

    let mut measurement_timer = time::interval(CONFIG.measurement_period);
    let enviro_phat = Arc::new(EnviroPHat::new(&CONFIG.i2c_bus_path).unwrap());

    let mut db_conn = SqliteConnection::establish(CONFIG.db_path.to_str().unwrap()).unwrap();

    loop {
        select! {
            _ = measurement_timer.tick() => {
                log::info!("Measuring");

                let phat = enviro_phat.clone();

                let measurement_task = task::spawn_blocking(move || {
                    phat.measure()
                });

                let measurement_res = measurement_task.await.unwrap().unwrap();
                log::info!("Measurement result: {measurement_res:?}");

                let measurements = {
                    use db::schema::measurements::dsl::*;

                    let insertable = InsertableMeasurement::from(measurement_res);
                    diesel::insert_into(measurements)
                        .values(&insertable)
                        .execute(&mut db_conn)
                          .unwrap();

                    measurements.load::<Measurement>(&mut db_conn).unwrap()
                };

                log::info!("Measurements already in DB: {measurements:?}");
            }
        }
    }
}
