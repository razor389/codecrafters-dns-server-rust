use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use byte_packet_buffer::BytePacketBuffer;
use header::ResultCode;
use packet::DnsPacket;
use query::{DnsQuestion, QueryType};
use anyhow::Result;
use std::env;

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

    // Bind to a UDP socket at port 2053
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");

    loop {
        let mut buffer = BytePacketBuffer::new();

        // Receive DNS query from the client
        let (amt, src) = udp_socket.recv_from(&mut buffer.buf)?;

        if amt > 0 {
            // Parse the incoming packet
            let packet = DnsPacket::from_buffer(&mut buffer)?;

            // Prepare the response packet
            let mut response_packet = DnsPacket::new();
            response_packet.header.id = packet.header.id;
            response_packet.header.response = true;
            response_packet.header.opcode = packet.header.opcode;
            response_packet.header.authoritative_answer = false;
            response_packet.header.truncated_message = false;
            response_packet.header.recursion_desired = packet.header.recursion_desired;
            response_packet.header.recursion_available = true;
            response_packet.header.z = false;
            response_packet.header.checking_disabled = packet.header.checking_disabled;
            response_packet.header.authed_data = packet.header.authed_data;

            // Check the opcode in the incoming query
            if packet.header.opcode != 0 {
                // Return NOTIMP (Not Implemented) for unsupported opcodes
                response_packet.header.rescode = ResultCode::NOTIMP;
                response_packet.questions = packet.questions.clone();
                if packet.questions.is_empty(){
                    response_packet.questions.push(DnsQuestion::new("codecrafters.io".to_string(), QueryType::A));
                }
                response_packet.header.questions = 1;
            } else if !packet.questions.is_empty() {
                // Process the questions only for standard queries (Opcode 0)
                for question in &packet.questions {
                    println!("Forwarding question: {:#?} to resolver: {}", question, resolver_addr);

                    // Forward each question individually
                    let mut resolver_packet = DnsPacket::new();
                    resolver_packet.questions.push(question.clone());
                    resolver_packet.header.id = packet.header.id; // Forward with the same ID

                    // Write the resolver packet to the buffer
                    let mut request_buffer = BytePacketBuffer::new();
                    resolver_packet.write(&mut request_buffer)?;

                    // Send the question to the resolver
                    let resolver_socket = UdpSocket::bind("0.0.0.0:0")?; // Ephemeral port
                    resolver_socket.send_to(&request_buffer.buf[0..request_buffer.pos], resolver_addr)?;

                    // Wait for the response from the resolver
                    let mut resolver_response_buffer = BytePacketBuffer::new();
                    let (response_size, _) = resolver_socket.recv_from(&mut resolver_response_buffer.buf)?;

                    // Parse the resolver's response
                    let resolver_response_packet = DnsPacket::from_buffer(&mut resolver_response_buffer)?;
                    
                    response_packet.questions.extend(resolver_response_packet.questions);
                    // Copy answers, authorities, and additional records from resolver's response
                    response_packet.answers.extend(resolver_response_packet.answers);
                    response_packet.authorities.extend(resolver_response_packet.authorities);
                    response_packet.resources.extend(resolver_response_packet.resources);
                }

                // Update header with the number of responses
                
                response_packet.header.answers = response_packet.answers.len() as u16;
                response_packet.header.authoritative_entries = response_packet.authorities.len() as u16;
                response_packet.header.resource_entries = response_packet.resources.len() as u16;

                // Set NOERROR if everything was successful
                response_packet.header.rescode = ResultCode::NOERROR;
                response_packet.header.questions = packet.questions.len() as u16;
            }
            
            // Write the response back to the client
            let mut response_buffer = BytePacketBuffer::new();
            response_packet.write(&mut response_buffer)?;

            println!("Sending response back to client at {}", src);
            udp_socket.send_to(&response_buffer.buf[0..response_buffer.pos], src)?;
        }
    }
}
