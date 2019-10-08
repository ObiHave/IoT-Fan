use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
fn main() {
    let mut value = 1;
    let step_path = "/sys/class/gpio/gpio45/value";
    let mut file = File::create(step_path).expect("Cannot open gpio file.");
    for _ in 0..800 {
        write!(file, "{}", value).expect("Cannot write to GPIO value file.");
        if value == 1 {
            value = 0;
        } else {
            value = 1;
        }
        thread::sleep(Duration::from_millis(50));
    }
}
