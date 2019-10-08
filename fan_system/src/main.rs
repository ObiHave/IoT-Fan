use std::net::UdpSocket;
use std::fs::File;
use std::io::Write;
use nom::*;
#[macro_use]
extern crate failure;
use failure::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
mod stepper;
mod temperature;
mod command;
use command::{CommandID, ControlMode, parse_message};
use std::process::Command;

fn main() {
    // Bind a UDP socket to listen for command messages from the Mobile App.
    let socket = UdpSocket::bind("192.168.8.1:8080").expect("Could not connect to UDP socket.");
    println!("Listening on {}", socket.local_addr().unwrap());
    // Begin oscillation thread, no-op to begin.
    let osc_mutex = Arc::new(Mutex::new(stepper::Oscillation::default()));
    let osc_main_mutex = osc_mutex.clone();
    // Begin the oscillation thread.
    thread::spawn(move || {
        stepper::oscillation_thread(osc_mutex);
    });

    let temp_mutex = Arc::new(Mutex::new(temperature::Temperature::default()));
    let temp_main_mutex = temp_mutex.clone();
    // Begin temperature measurement thread.
    thread::spawn(move || {
        temperature::maintain_temp(temp_mutex);
    });
    loop {
        let mut buffer = [0; 100];
        let (bytes, _addr) = socket.recv_from(&mut buffer).expect("Failed to recieve message.");

        let received = &mut buffer[..bytes];
        let (_, command) = match parse_message(&received) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };
        //If the new message is not a command to oscillate, disable oscillation.
        if command.cmd_id != CommandID::setControlMode && command.param_id != ControlMode::oscillate {
            let mut osc = osc_main_mutex.lock().expect("The oscillation mutex was poisoned!");
            osc.oscillation_enable = false;
        }
        if command.cmd_id != CommandID::temperatureSpeedMode {
            let mut tmp = temp_main_mutex.lock().expect("The temperature mutex was poisoned!");
            tmp.enable = false;
        }
        match command.cmd_id {
            CommandID::setControlMode => {
                print!("Set Control Mode:");
                match command.param_id {
                    ControlMode::sleep => {
                        println!(" Sleep.");
                        set_duty_cycle(0).expect("Cannot sleep the fan.");
                    },
                    ControlMode::tracking => {
                        println!(" Tracking.");
                    },
                    ControlMode::oscillate => {
                        println!(" Oscillate {:?} degrees.", command.param_value);
                        let mut osc = osc_main_mutex.lock().expect("The oscillation mutex was poisoned!");
                        osc.range = Some(command.param_value);
                        osc.oscillation_enable = true;
                    },
                    ControlMode::point => {
                        println!(" Point to {:?} degrees.", command.param_value);
                        if command.param_value <= 360 {
                            let mut osc = osc_main_mutex.lock().expect("The oscillation mutex was poisoned!");
                            osc.point(command.param_value.into());
                        } else {
                            println!("Invalid Point parameter {}, please enter one within 0 - 360 Degrees", command.param_value);
                        }
                    },
                    ControlMode::night => {
                        println!(" Night time: {:?} minute timer.", command.param_value);
                        let timer_begin = Instant::now();
                        thread::spawn(move || {
                            loop {
                                if timer_begin.elapsed().as_secs() / 60 >= command.param_value.into() {
                                    set_duty_cycle(0).expect("Cannot set the fan speed!");
                                    break;
                                }
                                thread::sleep(Duration::from_secs(5));
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
                println!("Temperature Mode: Desired Temp: {:?} Degrees F.", command.param_value);
                let mut tmp = temp_main_mutex.lock().expect("The temperature mutex was poisoned!");
                tmp.enable = true;
                tmp.desired = command.param_value;
            },
            CommandID::invalidCommand => {
                println!("Invalid command ID: {:?}", command.cmd_id);
            }
        }        
    }    
}

fn set_duty_cycle(percentage: u32) -> Result<u32, Error> {
    let duty = percentage * 10000;        
    let duty_path = "/sys/class/pwm/pwmchip4/pwm-4:0/duty_cycle";
    let mut file = File::create(duty_path)?;
    write!(file, "{}", duty)?;
    Ok(percentage)
}

#[derive(Fail, Debug, PartialEq)]
pub enum FanError {
    /// An error was thrown by the serial communication driver.
    #[fail(display = "Could not parse the command sent.")]
    ParseError,
    /// An error was thrown by the temperature sensor driver.
    #[fail(display = "Could not read the Temperature Sensor.")]
    TemperatureError,
}