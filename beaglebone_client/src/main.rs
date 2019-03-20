use std::net::UdpSocket;


fn main() {
    let client = UdpSocket::bind("192.168.8.1:8080").expect("Could not connect to 8.1:8080.");
    println!("Listening on {}", client.local_addr().unwrap());
    loop {
        let mut buffer = [0; 100];
        let (bytes, addr) = client.recv_from(&mut buffer).expect("Failed to recieve message.");
        let rx = &mut buffer[..bytes];
        println!("{} Bytes received from {}.\nMessage Received: {}", bytes, addr, String::from_utf8(rx.to_vec()).unwrap());
    }
}
