use super::*;
pub struct Temperature {
    pub enable: bool,
    pub desired: u16
}
impl Temperature {
    pub fn default() -> Temperature {
        Temperature {
            enable: false,
            desired: 70
        }
    }
}
fn from_slice_temperature(input: &[u8]) -> Result<u16, Error> {
    u16::from_str_radix(std::str::from_utf8(input)?, 16).map_err(|_| FanError::ParseError.into())
}
named!(
    parse_temp(&[u8]) -> u16,
    do_parse!(
        // Take off the '0x'
        take!(2) >>
        lsb: map_res!(take!(2), from_slice_temperature) >>
        msb: map_res!(take!(2), from_slice_temperature) >>
        take!(1) >> 
        (msb * 256 + lsb)
    )
);
pub fn current_temperature() -> Result<f64, FanError> {
    let cmd = Command::new("i2cget").arg("-y")
                                    .arg("2")
                                    .arg("0x18")
                                    .arg("0x05")
                                    .arg("w")
                                    .output()
                                    .expect("Could not do i2c.");
    if cmd.stderr.len() == 0 {
        let (_, parsed) = parse_temp(&cmd.stdout).expect("Could not parse the Temperature Readout.");
        let chopped = parsed & 0x0FFF;
        Ok((chopped as f64 / 16.0) * 9.0 / 5.0 + 32.0)
    } else {
        println!("{}", String::from_utf8(cmd.stderr).expect("Could not parse I2C error."));
        return Err(FanError::TemperatureError.into());
    }
}
pub fn maintain_temp(arg_mutex: Arc<Mutex<Temperature>>) {
    loop {
        {
            let arg = arg_mutex.lock().expect("The temperature mutex was poisoned!");
            if !arg.enable {
                continue
            }
            let current_temperature = current_temperature().expect("Could not read the temperature sensor.");
            if arg.desired >= current_temperature as u16 {
                set_duty_cycle(50).expect("Could not set fan speed.");
            } else {
                set_duty_cycle(100).expect("Could not set fan speed.");
            }
        }
    thread::sleep(Duration::from_secs(5));
    }
}