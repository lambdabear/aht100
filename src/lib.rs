use rppal::i2c::I2c;
use std::thread::sleep;
use std::time::Duration;

const ADDR: u16 = 0x38;
const CMD_INIT: u8 = 0xE1;
const INIT_ARG1: u8 = 0x08;
const INIT_ARG2: u8 = 0x00;
const CMD_MEASURE: u8 = 0xAC;
const MEASURE_ARG1: u8 = 0x33;
const MEASURE_ARG2: u8 = 0x00;
const CMD_RESET: u8 = 0xBA;

pub struct Aht100 {
    i2c: I2c,
}

pub struct AhtData {
    pub temp: f32,
    pub hum: f32,
}

pub struct AhtStatus {
    busy: bool,
    mode: Mode,
    cal: bool,
}

pub enum Mode {
    Nor,
    Cyc,
    Cmd,
}

impl Aht100 {
    pub fn new(mut i2c: I2c) -> Result<Self, ()> {
        match i2c.set_slave_address(ADDR) {
            Ok(_) => Ok(Self { i2c }),
            Err(e) => {
                println!("I2C set slave address failed. {}", e);
                Err(())
            }
        }
    }

    pub fn reset(&mut self) -> Result<(), ()> {
        match self.i2c.write(&[CMD_RESET]) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Write reset command failed. {}", e);
                Err(())
            }
        }
    }

    pub fn init(&mut self) -> Result<AhtStatus, ()> {
        sleep(Duration::from_millis(40));
        match self.i2c.write(&[CMD_INIT, INIT_ARG1, INIT_ARG2]) {
            Ok(_) => {
                sleep(Duration::from_millis(75));
                let mut buffer = [0; 6];
                match self.i2c.read(&mut buffer) {
                    Ok(len) if len > 0 => Ok(self.decode_status(buffer[0])),
                    Err(e) => {
                        println!("Read device status failed. {}", e);
                        Err(())
                    }
                    _ => {
                        println!("Read device status no response");
                        Err(())
                    }
                }
            }
            Err(e) => {
                println!("Write init command failed. {}", e);
                Err(())
            }
        }
    }

    pub fn measure(&mut self) -> Result<AhtData, ()> {
        match self.i2c.write(&[CMD_MEASURE, MEASURE_ARG1, MEASURE_ARG2]) {
            Ok(_) => {
                sleep(Duration::from_millis(75));
                let mut buffer = [0; 6];
                match self.i2c.read(&mut buffer) {
                    Ok(len) if len == 6 => {
                        let status = self.decode_status(buffer[0]);
                        if status.busy {
                            println!("Device is busy");
                            return Err(());
                        }
                        if !status.cal {
                            println!("Device is not calibration");
                            return Err(());
                        }
                        Ok(self
                            .decode_data([buffer[1], buffer[2], buffer[3], buffer[4], buffer[5]]))
                    }
                    Err(e) => {
                        println!("Read device status failed. {}", e);
                        Err(())
                    }
                    _ => {
                        println!("Read device status no response");
                        Err(())
                    }
                }
            }
            Err(e) => {
                println!("Write measure command failed. {}", e);
                Err(())
            }
        }
    }

    fn decode_status(&self, byte: u8) -> AhtStatus {
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

    fn decode_data(&self, bytes: [u8; 5]) -> AhtData {
        let hum = ((bytes[0] as u32) << 12) | ((bytes[1] as u32) << 4) | ((bytes[2] >> 4) as u32);
        let temp =
            (((bytes[2] << 4 >> 4) as u32) << 16) | ((bytes[3] as u32) << 8) | (bytes[4] as u32);
        let d = 1_u32 << 20;
        let hum: f32 = hum as f32 / d as f32 * 100.0;
        let temp: f32 = temp as f32 / d as f32 * 200.0 - 50.0;
        AhtData { hum, temp }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
