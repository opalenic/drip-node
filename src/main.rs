use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use tokio::{select, task, time};
use tokio_tungstenite;

use futures_util::{SinkExt, StreamExt};

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

mod enviro_phat;
use enviro_phat::{create_measurement_task, EnviroPHat};

use diesel::prelude::*;

mod db;

mod websocket_conn;
use websocket_conn::WebsocketConnection;

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
        dotenv::dotenv().map_err(|e| anyhow!(".env file load: {e}"))?;
        let i2c_bus_path = PathBuf::from(
            dotenv::var(Self::I2C_DEV_PATH_ENV_VAR)
                .map_err(|e| anyhow!("{} {}", Self::I2C_DEV_PATH_ENV_VAR, e))?,
        );

        let measurement_period_secs = dotenv::var(Self::MEASUREMENT_PERIOD_ENV_VAR)
            .map_err(|e| anyhow!("{} {}", Self::MEASUREMENT_PERIOD_ENV_VAR, e))?
            .parse()
            .map_err(|e| anyhow!("{} {}", Self::MEASUREMENT_PERIOD_ENV_VAR, e))?;

        let measurement_period = Duration::from_secs(measurement_period_secs);

        let db_path = PathBuf::from(
            dotenv::var(Self::DB_FILE_PATH_ENV_VAR)
                .map_err(|e| anyhow!("{} {}", Self::DB_FILE_PATH_ENV_VAR, e))?,
        );

        Ok(Self {
            i2c_bus_path,
            measurement_period,
            db_path,
        })
    }
}


#[derive(Debug, Serialize)]
enum OutgoingMsg {
    TypeA(u16),
    TypeB(u32),
}

#[derive(Debug, Deserialize)]
struct IncomingMsg(u8);


#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut measurement_timer = time::interval(CONFIG.measurement_period);

    let db_conn = Arc::new(Mutex::new(
        SqliteConnection::establish(CONFIG.db_path.to_str().unwrap()).unwrap(),
    ));
    let enviro_phat = Arc::new(EnviroPHat::new(&CONFIG.i2c_bus_path).unwrap());

    {
        let w: WebsocketConnection<OutgoingMsg, IncomingMsg> = WebsocketConnection::new("ws://127.0.0.1:8080/ws")
            .await
            .unwrap();

        log::trace!("{w:?}");

        for i in 0..10 {
            w.send(OutgoingMsg::TypeA(i));
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        tokio::time::sleep(Duration::from_secs(100)).await;
        log::error!("Closing websocket connection")
    }


    let (ws, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:8080/ws").await.unwrap();

    let (mut ws_tx, ws_rx) = ws.split();
    let mut ws_rx = ws_rx.fuse();
    let mut i = 0;
    loop {
        select! {
            _ = measurement_timer.tick() => {
                let res = task::spawn_blocking(
                    create_measurement_task(enviro_phat.clone(), db_conn.clone())
                )
                .await;

                if let Err(e) = res {
                    log::error!("Encountered error while measuring and persinting data to the DB: {e}");
                }
                i += 1;
                ws_tx.send(format!("Boom {i}").into()).await.unwrap();
                log::error!("Future done {i}");

            },
            Some(something) = ws_rx.next() => {
                log::error!("Got something {something:?}");
            }
        }
    }
}
