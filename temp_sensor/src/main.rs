use std::process::Command;
use nom::*;
use std::thread;
use std::time::Duration;
#[macro_use]
extern crate failure;
use failure::Error;

fn main() {
    loop {
        let cmd = Command::new("i2cget").arg("-y")
                                        .arg("2")
                                        .arg("0x18")
                                        .arg("0x05")
                                        .arg("w")
                                        .output()
                                        .expect("Could not do i2c.");
        //println!("Returned: {}, Error: {}", String::from_utf8(cmd.stdout).expect("Cannot"), String::from_utf8(cmd.stderr).expect("Cannot"));
        //let read = LittleEndian::read_i16(cmd.stdout.as_slice());
        if cmd.stderr.len() == 0 {
            let (_, parsed) = trash(&cmd.stdout).expect("Could not parse the Temperature Readout.");
            print!("Decimal: {}\nHex: {}", parsed, String::from_utf8(cmd.stdout).expect("Cannot"));
            let chopped = parsed & 0x0FFF;
            println!("Degrees C: {} | Degrees F: {}", chopped as f64 / 16.0, (chopped as f64 / 16.0) * 9.0 / 5.0 + 32.0);
        } else {
            println!("{}", String::from_utf8(cmd.stderr).expect("Nah."));
        }
        thread::sleep(Duration::from_secs(5));
    }
}
/// Converts a slice of 2 `u8`s into a `u16`.
pub fn from_slice(input: &[u8]) -> Result<u16, Error> {
    u16::from_str_radix(std::str::from_utf8(input)?, 16)
        .map_err(|_| FanError::ParseError.into())
}
named!(
    trash(&[u8]) -> u16,
    do_parse!(
        // Take off the '0x'
        take!(2) >>
        lsb: map_res!(take!(2), from_slice) >>
        msb: map_res!(take!(2), from_slice) >>
        take!(1) >> 
        (msb * 256 + lsb)
    )
);
#[derive(Fail, Debug, PartialEq)]
pub enum FanError {
    /// An error was thrown by the serial communication driver.
    #[fail(display = "The fan had an error.")]
    ParseError,
}