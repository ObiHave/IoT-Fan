use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

const DIRECTION_PATH: &'static str = "/sys/class/gpio/gpio44/value";
const STEP_PATH: &'static str = "/sys/class/gpio/gpio45/value";
const STEPSPERREV: f64 = 400.0;
const DEFAULT_POS: u16 = 400;

pub enum Direction {
    CW = 0,
    CCW = 1,
}
pub struct Oscillation {
    dir: Direction,
    pub oscillation_enable: bool,
    pub range: Option<u16>,
    position: u16,
}
impl Oscillation {
    pub fn default() -> Oscillation {
        reset_direction();
        Oscillation{
            dir: Direction::CW,
            oscillation_enable: false,
            range: None,
            position: DEFAULT_POS,
        }
    }
    pub fn point(&mut self, pos: f64) {
        // pos is the absolute angle to point. Convert the degrees to steps.
        let desired_pos = deg_to_step(pos);
        let steps;
        // Choose what direction in which to point.
        if self.position < desired_pos {
            self.set_direction(Direction::CW);
            steps = desired_pos - self.position;
        } else {
            self.set_direction(Direction::CCW);
            steps = self.position - desired_pos;
        }
        self.step(steps, true);
    }
    fn step(&mut self, steps: u16, inter_step_pause: bool) {
        let mut file = File::create(STEP_PATH).expect("Cannot open the STEP GPIO file!");
        for _ in 0..steps {
            write!(file, "1").expect("Cannot write to STEP GPIO file!");
            write!(file, "0").expect("Cannot write to STEP GPIO file!");
            match self.dir {
                Direction::CW => {
                    self.position += 1;
                },
                Direction::CCW => {
                    self.position -= 1;
                }
            }
            if inter_step_pause {
                thread::sleep(Duration::from_millis(50));
            }
        }
        println!("{}", self.position);
    }
    /*
    pub fn step_deg(&mut self, deg: f64) {
        self.step(deg_to_step(deg) as u16, false); 
    }
    */
    pub fn set_direction(&mut self, dir: Direction) {
        let mut file = File::create(DIRECTION_PATH).expect("Cannot open direction gpio!");
        self.dir = dir;
        match self.dir {
            Direction::CW => {
                write!(file, "{}", Direction::CW as u8).expect("Cannot write to Stepper Direction GPIO file.");
            },
            Direction::CCW => {
                write!(file, "{}", Direction::CCW as u8).expect("Cannot write to Stepper Direction GPIO file.");
            },
        }
    }
}
/*
fn steps_to_deg(step: f64) -> u16 {
    (step / STEPSPERREV * 360.0) as u16 + DEFAULT_POS
}
*/
fn deg_to_step(deg: f64) -> u16 {
    (deg / 360.0 * STEPSPERREV) as u16 + DEFAULT_POS
}
fn reset_direction() {
    let mut file = File::create(DIRECTION_PATH).expect("Cannot open direction gpio!");
    write!(file, "{}", Direction::CW as u8).expect("Cannot write to Stepper Direction GPIO file.");
}
pub fn oscillation_thread(motor: Arc<Mutex<Oscillation>>) {
    loop {
        {
            let mut osc = motor.lock().expect("The oscillation mutex was poisoned!");
            if !osc.oscillation_enable {
                continue
            }
            let (cw_limit, ccw_limit) = match osc.range {
                Some(x) => {
                    // Range is the range in degrees.
                    let step_range = deg_to_step(x as f64 / 2.0);
                    (Some(DEFAULT_POS + step_range), Some(DEFAULT_POS - step_range))
                },
                None => (None, None),
            };
            if let Some(data) = cw_limit {
                if osc.position >= data {
                    osc.set_direction(Direction::CW);
                }
            }
            if let Some(data) = ccw_limit {
                if osc.position <= data {
                    osc.set_direction(Direction::CW);
                }
            }
            osc.step(1, false);
        }
    thread::sleep(Duration::from_millis(50));
    }
}