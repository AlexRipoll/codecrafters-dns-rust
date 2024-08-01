use std::net::UdpSocket;

use clap::{arg, Command};
use field::{Class, QType};
use header::Header;
use packet::Packet;
use question::Question;
use resource_records::ResourceRecord;

mod field;
mod header;
mod packet;
mod question;
mod resource_records;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    let matches = Command::new("dns-rs")
        .version("1.0")
        .about("A simple Domain Name System server")
        .arg(arg!(--resolver <VALUE>).required(true))
        .get_matches();

    let resolver = matches.get_one::<String>("resolver").expect("required");

    let mut header = Header::default();

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                println!("init {:?}", buf);
                let mut packet = Packet::from_bytes(buf);
                println!("-->Header {:#?}", packet.header);

                // header = header
                //     .id(packet.header.id)
                //     .query_response(true)
                //     .opcode(packet.header.opcode)
                //     .recursion_desired(packet.header.recursion_desired)
                //     .response_code(if packet.header.opcode == 0 { 0 } else { 4 })
                //     .question_count(packet.header.question_count)
                //     .build();
                // println!("-->Header {:#?}", header);

                let forward_packets = packet.split();

                let mut dns = Dns::new(header.build());

                let resource_records = Dns::forward(&udp_socket, resolver, forward_packets);

                // TODO: check header counts

                let response = packet.merge();

                // dns.questions = packet.questions.clone();
                //
                // for i in 0..packet.header.question_count {
                //     let question = packet.questions.get(i as usize).unwrap();
                //     dns.add_resource_record(
                //         question.name.clone(),
                //         question.qtype,
                //         question.class,
                //         60,
                //         4,
                //         vec![8, 8, 8, 8],
                //     );
                //     dns.header.inc_ancount();
                // }
                //
                // let response = dns.response();

                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Dns {
    header: Header,
    questions: Vec<Question>,
    resource_records: Vec<ResourceRecord>,
}

impl Dns {
    fn new(header: Header) -> Self {
        Self {
            header,
            questions: Vec::new(),
            resource_records: Vec::new(),
        }
    }

    fn add_question(&mut self, name: String, qtype: QType, class: Class) {
        self.questions.push(Question::new(name, qtype, class));
    }

    fn add_resource_record(
        &mut self,
        name: String,
        qtype: QType,
        class: Class,
        ttl: u32,
        rdlength: u16,
        rdata: Vec<u8>,
    ) {
        self.resource_records.push(ResourceRecord::new(
            name, qtype, class, ttl, rdlength, rdata,
        ));
    }

    fn forward(
        udp_socket: &UdpSocket,
        addr: &String,
        packets: Vec<Vec<u8>>,
    ) -> Vec<ResourceRecord> {
        let mut responses: Vec<ResourceRecord> = Vec::new();

        for packet in packets {
            println!("--> Packet {:?}", packet);
            udp_socket
                .send_to(&packet, addr)
                .expect("Error receiving data: {}");

            let mut response_buf = [0u8; 512];
            udp_socket
                .recv_from(&mut response_buf)
                .expect("Failed to receive response from upstream");

            let rr = ResourceRecord::from_bytes(&response_buf[..]);

            responses.push(rr);
        }

        responses
    }
}

fn labels_from_bytes(data: &[u8], start_pos: usize) -> (String, usize) {
    let mut cursor = start_pos;
    let mut labels: Vec<String> = Vec::new();

    while data[cursor] != 0 {
        if data[cursor] & 0b11000000 == 0b11000000 {
            let offset =
                (((data[cursor] & 0b00111111) as u16) << 8 | data[cursor + 1] as u16) as usize;
            let (label, _) = labels_from_bytes(&data, offset);
            labels.push(label);
            // increase the index by tw0 since the offset used 2 bytes
            cursor += 2;
        } else {
            let length = data[cursor] as usize;
            cursor += 1;
            let label = std::str::from_utf8(&data[cursor..cursor + length]).unwrap();
            labels.push(label.to_string());
            cursor += length;
        }
    }

    (labels.join("."), cursor)
}
