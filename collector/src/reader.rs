use dht11::Dht11;
use rppal::gpio::Gpio;
use rppal::hal::Delay;
use std::thread::sleep;
use std::time::Duration;

pub fn read_dht11(pin: u8) -> Result<(i16, u16), Box<dyn std::error::Error>> {
    let my_pin = Gpio::new()?.get(pin)?.into_io(rppal::gpio::Mode::Output);
    let mut dht11 = Dht11::new(my_pin);

    let mut delay = Delay::new();

    loop {
        match dht11.perform_measurement(&mut delay) {
            Ok(res) => {
                return Ok((res.temperature, res.humidity));
            }
            Err(err) => {
                println!("Error reading retrying.., {:?}", err);
                sleep(Duration::from_millis(1000));
            }
        }
    }
}
