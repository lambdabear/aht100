#![no_std]

use defmt::Format;
use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::{Read, Write},
};

const ADDR: u8 = 0x38;
const CMD_INIT: u8 = 0xE1;
const INIT_ARG1: u8 = 0x08;
const INIT_ARG2: u8 = 0x00;
const CMD_MEASURE: u8 = 0xAC;
const MEASURE_ARG1: u8 = 0x33;
const MEASURE_ARG2: u8 = 0x00;
const CMD_RESET: u8 = 0xBA;

pub struct Aht100<I2C> {
    i2c: I2C,
}

#[derive(Format)]
pub struct AhtData {
    pub temp: f32,
    pub hum: f32,
}

impl AhtData {
    pub fn from_bytes(bytes: [u8; 5]) -> Self {
        let hum = ((bytes[0] as u32) << 12) | ((bytes[1] as u32) << 4) | ((bytes[2] >> 4) as u32);
        let temp =
            (((bytes[2] << 4 >> 4) as u32) << 16) | ((bytes[3] as u32) << 8) | (bytes[4] as u32);
        let d = 1_u32 << 20;
        let hum: f32 = hum as f32 / d as f32 * 100.0;
        let temp: f32 = temp as f32 / d as f32 * 200.0 - 50.0;
        AhtData { hum, temp }
    }
}

pub struct AhtStatus {
    pub busy: bool,
    pub mode: Mode,
    pub cal: bool,
}

impl AhtStatus {
    fn from_byte(byte: u8) -> Self {
        AhtStatus {
            busy: byte > 0x7F,
            mode: match byte & 0b01100000 {
                0x00 => Mode::Nor,
                0x20 => Mode::Cyc,
                _ => Mode::Cmd,
            },
            cal: (byte & 0x08) == 0x08,
        }
    }
}

pub enum Mode {
    Nor,
    Cyc,
    Cmd,
}

impl<I2C> Aht100<I2C>
where
    I2C: Read + Write,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn reset(&mut self) -> Result<(), ()> {
        match self.i2c.write(ADDR, &[CMD_RESET]) {
            Ok(_) => Ok(()),
            Err(_e) => Err(()),
        }
    }

    pub fn init(&mut self, delay: &mut dyn DelayMs<u16>) -> Result<AhtStatus, ()> {
        delay.delay_ms(40_u16);
        match self.i2c.write(ADDR, &[CMD_INIT, INIT_ARG1, INIT_ARG2]) {
            Ok(_) => {
                delay.delay_ms(75_u16);
                let mut buffer = [0; 6];
                match self.i2c.read(ADDR, &mut buffer) {
                    Ok(_) => Ok(AhtStatus::from_byte(buffer[0])),
                    Err(_) => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    }

    pub fn measure(&mut self, delay: &mut dyn DelayMs<u16>) -> Result<[u8; 5], ()> {
        match self
            .i2c
            .write(ADDR, &[CMD_MEASURE, MEASURE_ARG1, MEASURE_ARG2])
        {
            Ok(_) => {
                delay.delay_ms(75_u16);
                let mut buffer = [0; 6];
                match self.i2c.read(ADDR, &mut buffer) {
                    Ok(_) => {
                        let status = AhtStatus::from_byte(buffer[0]);
                        if status.busy {
                            return Err(());
                        }
                        if !status.cal {
                            return Err(());
                        }
                        Ok([buffer[1], buffer[2], buffer[3], buffer[4], buffer[5]])
                    }
                    Err(_e) => Err(()),
                }
            }
            Err(_e) => Err(()),
        }
    }

    pub fn free(self) -> I2C {
        self.i2c
    }
}
