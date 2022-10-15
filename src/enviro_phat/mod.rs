use anyhow::Result;

#[cfg(feature = "enviro_phat_v1")]
mod v1;
#[cfg(feature = "enviro_phat_v1")]
pub use v1::EnviroPHatV1 as EnviroPHat;

#[cfg(feature = "enviro_phat_stub")]
mod stub;
#[cfg(feature = "enviro_phat_stub")]
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
