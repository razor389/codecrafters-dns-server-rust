use std::{env, net::{Ipv4Addr, SocketAddr, UdpSocket}};
use byte_packet_buffer::BytePacketBuffer;
use header::ResultCode;
use packet::DnsPacket;
use query::{DnsQuestion, QueryType};
use record::DnsRecord;
use anyhow::Result;
mod header;
mod byte_packet_buffer;
mod packet;
mod record;
mod query;

fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

     // Parse command line arguments
     let args: Vec<String> = env::args().collect();
     let resolver_addr = if args.len() == 3 && args[1] == "--resolver" {
         args[2].parse::<SocketAddr>().expect("Invalid resolver address")
     } else {
         panic!("Usage: ./your_server --resolver <ip:port>");
     };

    // Bind to the UDP socket at the specified address (port 2053)
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");

    
    loop {
        let mut buffer = BytePacketBuffer::new();

        // Receive data from the socket
        let (amt, src) = udp_socket.recv_from(&mut buffer.buf)?;
        let packet = DnsPacket::from_buffer(&mut buffer)?;
        //println!("header: {:#?}", packet.header);

        // for q in packet.questions {
        //     println!("questions: {:#?}", q);
        // }
        // for rec in packet.answers {
        //     println!("answers: {:#?}", rec);
        // }
        // for rec in packet.authorities {
        //     println!("authorities: {:#?}", rec);
        // }
        // for rec in packet.resources {
        //     println!("resources: {:#?}", rec);
        // }
        if amt> 0 {
            let mut response_packet = DnsPacket::new();
            let qtype = QueryType::A;
            let addr = Ipv4Addr::new(8, 8, 8, 8);

            response_packet.header.id = packet.header.id;
            response_packet.header.response = true;
            response_packet.header.opcode = packet.header.opcode;
            response_packet.header.authoritative_answer = false;
            response_packet.header.truncated_message = false;
            response_packet.header.recursion_desired = packet.header.recursion_desired;
            response_packet.header.recursion_available = false;
            response_packet.header.z = false;
            response_packet.header.checking_disabled = false;
            response_packet.header.authed_data = false;
            if response_packet.header.opcode == 0{
                response_packet.header.rescode = ResultCode::NOERROR;
            }
            else{
                response_packet.header.rescode = ResultCode::NOTIMP;
            }
            response_packet.header.questions = 0;
            response_packet.header.answers = 0;
            response_packet.header.authoritative_entries = 0;
            response_packet.header.resource_entries = 0;
            for question in packet.questions{
                let qname = question.name;
                response_packet.questions.push(DnsQuestion::new(qname.to_string(), qtype));
                response_packet.answers.push(DnsRecord::new_a(qname.to_string(), addr, 60));
            }

            let mut res_buffer = BytePacketBuffer::new();
            response_packet.write(&mut res_buffer)?;
            println!("response header: {:#?}", response_packet.header);

            udp_socket.send_to(&res_buffer.buf[0..res_buffer.pos], &src)?;
            
            println!("Received data from {} and sent response", src);
        }
    }
}
