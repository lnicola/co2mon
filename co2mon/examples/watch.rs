use co2mon::{Measurement, Result, Sensor};
use std::thread;
use std::time::Duration;

fn read_both(sensor: &Sensor) -> Result<(f32, u16)> {
    let mut temperature = None;
    let mut co2 = None;
    loop {
        match sensor.read()? {
            Measurement::Temperature(val) => temperature = Some(val),
            Measurement::CO2(val) => co2 = Some(val),
            _ => {}
        }
        if let (Some(temperature), Some(co2)) = (temperature, co2) {
            return Ok((temperature, co2));
        }
    }
}

fn main() -> Result<()> {
    let sensor = Sensor::open_default()?;
    loop {
        match read_both(&sensor) {
            Ok((temperature, co2)) => println!("{:.4} °C, {} ppm CO₂", temperature, co2),
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(5));
    }
}
