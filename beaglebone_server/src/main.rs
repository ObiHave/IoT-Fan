use std::net::UdpSocket;
use std::io;

fn main() {
    let server = UdpSocket::bind("192.168.8.52:8081").unwrap();
    println!("Sending on {}", server.local_addr().unwrap());
    loop {
        let mut msg = String::new();
        println!("Enter a message: ");
        io::stdin().read_line(&mut msg).expect("Failed to read a line.");
        println!("Sending: {}", msg);
        let bytes = server.send_to(msg.as_bytes(), "192.168.8.1:8080").expect("Failed to send message.");
        println!("{} bytes sent.\nMessage: {} sent.", bytes, msg);
    }
}
