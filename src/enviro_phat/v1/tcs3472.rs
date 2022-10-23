use anyhow::{anyhow, Result};

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CBus, LinuxI2CMessage};

use std::sync::{Arc, Mutex};

use super::LightLevel;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Gain {
    Mult1X = 0b00,
    Mult4X = 0b01,
    Mult16X = 0b10,
    Mult60X = 0b11,
}

#[derive(Debug)]
pub struct Tcs3472 {
    comm_channel: Arc<Mutex<LinuxI2CBus>>,
}

impl Tcs3472 {
    const I2C_ADDR: u16 = 0x77;

    const CMD_REG_MASK: u8 = 0x80;
    const CMD_REG_AUTOINCREMENT: u8 = 0x20;

    const ENABLE_REG_ADDR: u8 = 0x00;
    const ENABLE_REG_AEN: u8 = 0x02;
    const ENABLE_REG_PON: u8 = 0x01;

    #[allow(dead_code)]
    const TIMING_REG_ADDR: u8 = 0x01;
    #[allow(dead_code)]
    const TIMING_REG_STEP_MS: f32 = 2.4;

    const CONTROL_REG_ADDR: u8 = 0x0f;

    const CHIP_ID_REG_ADDR: u8 = 0x12;
    const CHIP_ID_EXPECTED: u8 = 0x44; // 0x4d

    const CLEAR_DATA_REG_ADDR: u8 = 0x14;
    const CLEAR_DATA_REG_SIZE: usize = 2;

    pub fn new(comm_channel: Arc<Mutex<LinuxI2CBus>>) -> Result<Tcs3472> {
        // Check we have the correct sensor
        let mut id_data = [0];
        let mut id_msgs = [
            LinuxI2CMessage::write(&[Self::CMD_REG_MASK | Self::CHIP_ID_REG_ADDR])
                .with_address(Self::I2C_ADDR),
            LinuxI2CMessage::read(&mut id_data).with_address(Self::I2C_ADDR),
        ];

        log::debug!("Reading out chip ID");
        comm_channel.lock().unwrap().transfer(&mut id_msgs)?;

        log::debug!("Chip ID is {}", id_data[0]);

        if id_data[0] != Self::CHIP_ID_EXPECTED {
            return Err(anyhow!(
                "Wrong chip ID response at I2C address {:#2x}. Expected {:#2x} and got {:#2x}.",
                Self::I2C_ADDR,
                Self::CHIP_ID_EXPECTED,
                id_data[0]
            ));
        }

        log::debug!("Configuring TCS3472.");
        // Configure the sensor
        // Continuous integration at 1x Gain, 64 periods per integration (total time 154ms)
        let cmd_reg_enable_autoinc =
            Self::CMD_REG_MASK | Self::CMD_REG_AUTOINCREMENT | Self::ENABLE_REG_ADDR;
        let enable_reg = Self::ENABLE_REG_AEN | Self::ENABLE_REG_PON;

        let period_count = 64;
        let timing_reg = u8::MAX - period_count;

        let cmd_reg_control = Self::CMD_REG_MASK | Self::CONTROL_REG_ADDR;
        let control_reg = Gain::Mult1X as u8;

        let mut config_msgs = [
            LinuxI2CMessage::write(&[cmd_reg_enable_autoinc, enable_reg, timing_reg])
                .with_address(Self::I2C_ADDR),
            LinuxI2CMessage::write(&[cmd_reg_control, control_reg]).with_address(Self::I2C_ADDR),
        ];

        comm_channel.lock().unwrap().transfer(&mut config_msgs)?;

        Ok(Tcs3472 { comm_channel })
    }

    pub fn query_light_level(&self) -> Result<LightLevel> {
        let cmd_reg_read_color_autoinc =
            Self::CMD_REG_MASK | Self::CMD_REG_AUTOINCREMENT | Self::CLEAR_DATA_REG_ADDR;

        let mut read_data_buf = [0; Self::CLEAR_DATA_REG_SIZE];

        let mut read_data_msgs = [
            LinuxI2CMessage::write(&[cmd_reg_read_color_autoinc]).with_address(Self::I2C_ADDR),
            LinuxI2CMessage::read(&mut read_data_buf).with_address(Self::I2C_ADDR),
        ];

        self.comm_channel
            .lock()
            .unwrap()
            .transfer(&mut read_data_msgs)?;

        let raw_val: u16 = ((read_data_buf[1] as u16) << 8) | (read_data_buf[0] as u16);

        Ok(LightLevel((raw_val as f32) / (u16::MAX as f32)))
    }
}
