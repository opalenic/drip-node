mod bmp280;
mod tcs3472;

use anyhow::Result;
use i2cdev::linux::LinuxI2CBus;

use bmp280::{Bmp280, IIRCoeficient, Mode, Oversampling, StandbyTime};
use tcs3472::Tcs3472;

use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{LightLevel, Pressure, Temperature};
use super::{MeasureEnvironment, Measurement};

#[derive(Debug)]
pub struct EnviroPHatV1 {
    bmp: Bmp280,
    tcs: Tcs3472,
}

impl EnviroPHatV1 {
    pub fn new(i2c_bus_path: &Path) -> Result<EnviroPHatV1> {
        let i2c_bus = LinuxI2CBus::new(i2c_bus_path)?;
        let comm_channel = Arc::new(Mutex::new(i2c_bus));

        let bmp = bmp280::Bmp280::new(
            comm_channel.clone(),
            StandbyTime::Time1000ms,
            IIRCoeficient::Mult4X,
            Oversampling::Mult16X,
            Oversampling::Mult2X,
            Mode::Normal,
        )?;

        let tcs = tcs3472::Tcs3472::new(comm_channel)?;

        Ok(EnviroPHatV1 { bmp, tcs })
    }
}

impl MeasureEnvironment for EnviroPHatV1 {
    fn measure(&self) -> Result<Measurement> {
        let (pressure, temperature) = self.bmp.query_press_and_temp()?;
        let light_level = self.tcs.query_light_level()?;

        Ok(Measurement {
            pressure,
            temperature,
            light_level,
        })
    }
}
