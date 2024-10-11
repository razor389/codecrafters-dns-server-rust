use core::str;
use byte_packet_buffer::BytePacketBuffer;
use header::ResultCode;
use packet::DnsPacket;
use query::{DnsQuestion, QueryType};
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
        println!("header: {:#?}", packet.header);

        for q in packet.questions {
            println!("questions: {:#?}", q);
        }
        for rec in packet.answers {
            println!("answers: {:#?}", rec);
        }
        for rec in packet.authorities {
            println!("authorities: {:#?}", rec);
        }
        for rec in packet.resources {
            println!("resources: {:#?}", rec);
        }
        if amt> 0 {
            let mut response_packet = DnsPacket::new();
            let qname = "codecrafters.io";
            let qtype = QueryType::A;

            response_packet.header.id = 1234;
            response_packet.header.response = true;
            response_packet.header.opcode = 0;
            response_packet.header.authoritative_answer = false;
            response_packet.header.truncated_message = false;
            response_packet.header.recursion_desired = false;
            response_packet.header.recursion_available = false;
            response_packet.header.z = false;
            response_packet.header.checking_disabled = false;
            response_packet.header.authed_data = false;
            response_packet.header.rescode = ResultCode::NOERROR;
            response_packet.header.questions = 0;
            response_packet.header.answers = 0;
            response_packet.header.authoritative_entries = 0;
            response_packet.header.resource_entries = 0;
            response_packet.questions.push(DnsQuestion::new(qname.to_string(), qtype));
            
            let mut res_buffer = BytePacketBuffer::new();
            response_packet.write(&mut res_buffer)?;
            println!("{:#?}", response_packet.header);

            udp_socket.send_to(&res_buffer.buf[0..res_buffer.pos], &src).await?;
            
            println!("Received data from {} and sent response", src);
        }
    }
}
