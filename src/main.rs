use core::str;
use byte_packet_buffer::BytePacketBuffer;
use packet::DnsPacket;
use tokio::net::UdpSocket;
use anyhow::Result;
mod header;
mod byte_packet_buffer;
mod packet;
mod record;
mod query;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    // Bind to the UDP socket at the specified address (port 2053)
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").await.expect("Failed to bind to address");

    let mut buffer = BytePacketBuffer::new();

    loop {
        // Receive data from the socket
        let (amt, src) = udp_socket.recv_from(&mut buffer.buf).await?;
        let packet = DnsPacket::from_buffer(&mut buffer)?;
        println!("{:#?}", packet.header);

        for q in packet.questions {
            println!("{:#?}", q);
        }
        for rec in packet.answers {
            println!("{:#?}", rec);
        }
        for rec in packet.authorities {
            println!("{:#?}", rec);
        }
        for rec in packet.resources {
            println!("{:#?}", rec);
        }

        if amt> 0 {
            udp_socket.send_to(&buffer.buf, &src).await?;
            
            println!("Received data from {} and sent response", src);
        }
    }
}
