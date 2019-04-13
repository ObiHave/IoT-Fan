use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use super::*;
use std::sync::{Arc, Mutex};

const DIRECTION_PATH: &'static str = "/sys/class/gpio/gpio44/value";
const STEP_PATH: &'static str = "/sys/class/gpio/gpio45/value";
const STEPSPERREV: f64 = 400.0;
const DEFAULT_POS: u16 = 400;

pub enum direction {
    CW = 0,
    CCW = 1,
}
pub struct oscillation {
    dir: direction,
    range: Option<u16>,
    position: u16,
}
impl oscillation {
    pub fn default() -> oscillation {
        reset_direction();
        oscillation{
            dir: direction::CW,
            range: None,
            position: DEFAULT_POS,
        }
    }
    pub fn point(&mut self, pos: f64) {
        // pos is the absolute angle to point. Convert the degrees to steps.
        let desired_pos = self.deg_to_step(pos);
        let steps;
        // Choose what direction in which to point.
        if self.position < desired_pos {
            self.set_direction(direction::CW);
            steps = desired_pos - self.position;
        } else {
            self.set_direction(direction::CCW);
            steps = self.position - desired_pos;
        }
        self.step(steps);
    }
    fn step(&mut self, steps: u16) {
        let mut file = File::create(STEP_PATH).expect("Cannot open the STEP GPIO file!");
        for _ in 0..steps {
            write!(file, "1").expect("Cannot write to STEP GPIO file!");
            write!(file, "0").expect("Cannot write to STEP GPIO file!");
            match self.dir {
                direction::CW => {
                    self.position += 1;
                },
                direction::CCW => {
                    self.position -= 1;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        println!("{}", self.position);
    }
    pub fn step_deg(&mut self, deg: f64) {
        self.step(self.deg_to_step(deg) as u16); 
    }
    pub fn set_direction(&mut self, dir: direction) {
        let mut file = File::create(DIRECTION_PATH).expect("Cannot open direction gpio!");
        self.dir = dir;
        match self.dir {
            direction::CW => {
                write!(file, "{}", direction::CW as u8).expect("Cannot write to Stepper Firection GPIO file.");
            },
            direction::CCW => {
                write!(file, "{}", direction::CCW as u8).expect("Cannot write to Stepper Direction GPIO file.");
            },
        }
    }
    fn steps_to_deg(&self, step: f64) -> u16 {
        (step / STEPSPERREV * 360.0) as u16 + DEFAULT_POS
    }
    fn deg_to_step(&self, deg: f64) -> u16 {
        (deg / 360.0 * STEPSPERREV) as u16 + DEFAULT_POS
    }
}
fn reset_direction() {
    let mut file = File::create(DIRECTION_PATH).expect("Cannot open direction gpio!");
    write!(file, "{}", direction::CW as u8).expect("Cannot write to Stepper Firection GPIO file.");
}
/*
pub fn oscillation_thread(setting: Arc<Mutex<oscillation>>) {
    let mut file = File::create(DIRECTION_PATH).expect("Cannot open direction gpio!");
    loop {
        let osc = setting.lock().expect("The oscillation mutex was poisoned!");
        match osc.dir {
            direction::CW => {
                write!(file, "{}", direction::CW as u8).expect("Cannot write to Stepper Firection GPIO file.");
            },
            direction::CCW => {
                write!(file, "{}", direction::CCW as u8).expect("Cannot write to Stepper Direction GPIO file.");
            },
        }
        match osc.range {
            Some(x) => {

            }
            None => (),
        }
    }
}
*/