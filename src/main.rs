use std::io;
use tokio::net::UdpSocket;
#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Logs from your program will appear here!");

    // Bind to the UDP socket at the specified address (port 2053)
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").await.expect("Failed to bind to address");

    let mut buf = [0; 512];

    loop {
        // Receive data from the socket
        let (amt, src) = udp_socket.recv_from(&mut buf).await?;
        
        // Redeclare `buf` as slice of the received data and send reverse data back to the origin
        let buf = &mut buf[..amt];
        buf.reverse();
        
        // Send the reversed data back to the source
        udp_socket.send_to(buf, &src).await?;
        
        println!("Received data from {} and sent response", src);
    }
}
