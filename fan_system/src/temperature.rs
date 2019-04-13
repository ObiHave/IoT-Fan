use super::*;

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
pub fn temp_thread(mutex: Arc<Mutex<f64>>) {
    loop {
        let cmd = Command::new("i2cget").arg("-y")
                                        .arg("2")
                                        .arg("0x18")
                                        .arg("0x05")
                                        .arg("w")
                                        .output()
                                        .expect("Could not do i2c.");
        if cmd.stderr.len() == 0 {
            let mut temp = mutex.lock().expect("The temp mutex was poisoned!");
            let (_, parsed) = parse_temp(&cmd.stdout).expect("Could not parse the Temperature Readout.");
            //print!("Decimal: {}\nHex: {}", parsed, String::from_utf8(cmd.stdout).expect("Cannot"));
            let chopped = parsed & 0x0FFF;
            *temp = (chopped as f64 / 16.0) * 9.0 / 5.0 + 32.0;
        } else {
            println!("{}", String::from_utf8(cmd.stderr).expect("Could not parse I2C error."));
        }
        thread::sleep(Duration::from_millis(2000));
    }
}