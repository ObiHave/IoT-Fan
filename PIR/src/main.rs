use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Read;
fn main() {
    let step_path = "/sys/class/gpio/gpio5/value";
    
    loop {
        let mut file = File::open(step_path).expect("Cannot open gpio file.");
        let mut buffer = [0];
        file.read(&mut buffer).expect("Cannot read GPIO value file.");
        println!("{:?}", buffer[0] - 48);
        thread::sleep(Duration::from_millis(500));
    }
}