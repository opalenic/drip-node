use anyhow::Result;

use std::path::Path;

use super::{LightLevel, Pressure, Temperature};
use super::{MeasureEnvironment, Measurement};

#[derive(Debug)]
pub struct EnviroPHatStub(());

impl EnviroPHatStub {
    pub fn new(_i2c_bus_path: &Path) -> Result<EnviroPHatStub> {
        Ok(EnviroPHatStub(()))
    }
}

impl MeasureEnvironment for EnviroPHatStub {
    fn measure(&self) -> Result<Measurement> {
        let pressure = Pressure(101325.0);
        let temperature = Temperature(24.0);
        let light_level = LightLevel(2.4);

        Ok(Measurement {
            pressure,
            temperature,
            light_level,
        })
    }
}
