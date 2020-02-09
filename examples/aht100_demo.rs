use aht100::Aht100;
use rppal::i2c::I2c;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let i2c = I2c::new().expect("Init I2C failed");
    let mut dev = Aht100::new(i2c).expect("Init AHT100 device failed");

    if let Ok(_status) = dev.init() {
        loop {
            if let Ok(data) = dev.measure() {
                println!("Temp: {}, Hum: {}", data.temp, data.hum);
            } else {
                println!("Measure tempture and humidity failed");
                break;
            }
            sleep(Duration::from_secs(1));
        }
    }
}
