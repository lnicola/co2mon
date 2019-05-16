use co2mon::{Result, Sensor};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let sensor = Sensor::open_default()?;
    loop {
        match sensor.read() {
            Ok(reading) => println!(
                "{:.4} °C, {} ppm CO₂",
                reading.temperature(),
                reading.co2()
            ),
            Err(e) => eprintln!("{}", e),
        }
        thread::sleep(Duration::from_secs(5));
    }
}
