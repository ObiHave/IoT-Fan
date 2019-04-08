use std::net::UdpSocket;
use std::fs::File;
use std::io::Write;
use nom::*;
#[macro_use]
extern crate failure;
use failure::Error;
use std::thread;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub struct Command {
    cmd_id: CommandID,
    param_id: ControlMode,
    param_value: u16,
}
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CommandID {
    setControlMode = 0,
    percentSpeedMode = 1,
    temperatureSpeedMode = 2,
    invalidCommand,
}
impl From<u16> for CommandID {
    fn from(cmdid: u16) -> CommandID {
        match cmdid {
            0 => CommandID::setControlMode,
            1 => CommandID::percentSpeedMode,
            2 => CommandID::temperatureSpeedMode,
            _ => CommandID::invalidCommand,
        }
    }
}
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ControlMode {
    sleep = 0,
    tracking = 1,
    oscillate = 2,
    point = 3,
    night = 4,
    invalidParameter,
}
impl From<u16> for ControlMode {
    fn from(param: u16) -> ControlMode {
        match param {
            0 => ControlMode::sleep,
            1 => ControlMode::tracking,
            2 => ControlMode::oscillate,
            3 => ControlMode::point,
            4 => ControlMode::night,
            _ => ControlMode::invalidParameter,
        }
    }
}


named!(
    parse_message(&[u8]) -> Command,
    do_parse!(
        // Take command ID
        cmd_id: map_res!(take_until!(","), from_slice) >>
        // Take comma.
        take!(1) >>
        // Take parameter ID.
        ControlMode: map_res!(take_until!(","), from_slice) >>
        // Take the comma.
        take!(1) >>
        // Take the parameter value.
        param_val: map_res!(take_until!("."), from_slice) >>
        (Command {
            cmd_id: CommandID::from(cmd_id),
            param_id: ControlMode::from(ControlMode),
            param_value: param_val,
        })
    )
);
fn from_slice(input: &[u8]) -> Result<u16, Error> {
    u16::from_str_radix(std::str::from_utf8(input)?, 10).map_err(|_| FanError::ParseError.into())
}
fn main() {
    println!("Initializing!");
    let (kill_init,rx) = channel();
    thread::spawn(move || {
        let mut message = rx.recv_timeout(Duration::from_millis(50));
        while message.is_err() {   
                for i in 1..100 {
                    message = rx.recv_timeout(Duration::from_millis(1));
                    if message.is_ok() {
                        break;
                    }
                    set_duty_cycle(i).expect("Cannot set duty cycle.");
                    thread::sleep(Duration::from_millis(49));
                }
                if message.is_ok(){
                    break;
                }
                for x in (1..100).rev() {
                    message = rx.recv_timeout(Duration::from_millis(1));
                    if message.is_ok() {
                        break;
                    }
                    set_duty_cycle(x).expect("Cannot set duty cycle.");
                    thread::sleep(Duration::from_millis(49));
                }            
        }
    });
    
    let socket = UdpSocket::bind("192.168.8.1:8080").expect("Could not connect to UDP socket.");
    println!("Listening on {}", socket.local_addr().unwrap());
    let mut first = true;
    loop {
        let mut buffer = [0; 100];
        let (bytes, _addr) = socket.recv_from(&mut buffer).expect("Failed to recieve message.");
        if first {
            kill_init.send(1).expect("Unable to kill intialization.");
            first = false;
        }
        let received = &mut buffer[..bytes];
        let (_, command) = match parse_message(&received) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };
        match command.cmd_id {
            CommandID::setControlMode => {
                print!("Set Control Mode:");
                match command.param_id {
                    ControlMode::sleep => {
                        println!(" Sleep.");
                        set_duty_cycle(0);
                    },
                    ControlMode::tracking => {
                        println!(" Tracking.");
                    },
                    ControlMode::oscillate => {
                        println!(" Oscillate {:?} degrees.", command.param_value);                        
                    },
                    ControlMode::point => {
                        println!(" Point to {:?} degrees.", command.param_value);
                    },
                    ControlMode::night => {
                        println!(" Night time: {:?} minute timer.", command.param_value);
                        let timer_begin = Instant::now();
                        thread::spawn(move || {
                            if timer_begin.elapsed().as_secs() / 60 >= command.param_value {
                                set_duty_cycle(0);
                            }
                        });
                    },
                    ControlMode::invalidParameter => {
                        println!(" Invalid mode setting: {:?}", command.param_id);
                    },
                };
            },
            CommandID::percentSpeedMode => {
                println!("Speed Mode: {:?}%", command.param_value);
                let duty = command.param_value;
                if duty <= 100 {
                    set_duty_cycle(duty.into()).expect("Could not set fan speed.");
                    println!("Set fan speed to {:?}%.", duty); 
                } else {
                    println!("Please enter a valid fan speed percentage.");
                }
            },
            CommandID::temperatureSpeedMode => {
                println!("Temperature Mode: {:?} Degrees.", command.param_value);
            },
            CommandID::invalidCommand => {
                println!("Invalid command ID: {:?}", command.cmd_id);
            }
        }
        //println!("{} Bytes received from {}.\nMessage Received: {}", bytes, addr, msg);
        //let mut duty = String::new();
        //println!("Enter new Duty Cycle (%):");
        //io::stdin().read_line(&mut duty);
        
    }
    //let chip = PwmChip::new(4).expect("Couldnt get the chip.");
    //let pin = Pwm::new(4, 0).expect("Could not open the pin.");
    //println!("{}", pin.get_duty_cycle_ns().expect("cannot get duty cycle."));
}

fn set_duty_cycle(percentage: u32) -> Result<u32, Error> {
    let duty = percentage * 10000;        
    let duty_path = "/sys/class/pwm/pwmchip4/pwm-4:0/duty_cycle";
    let mut file = File::create(duty_path)?;
    let output_duty = 1000000 - duty;
    write!(file, "{}", output_duty)?;
    Ok(percentage)
}

#[derive(Fail, Debug, PartialEq)]
pub enum FanError {
    /// An error was thrown by the serial communication driver.
    #[fail(display = "The fan had an error.")]
    ParseError,
}