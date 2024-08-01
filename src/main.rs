use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    usize,
};

use clap::{arg, Command};
use header::Header;

mod header;

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

                dns.questions = packet.questions;
                dns.resource_records = resource_records;

                let response = dns.merge();

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

    fn add_question(&mut self, name: String, qtype: QueryType, class: Class) {
        self.questions.push(Question::new(name, qtype, class));
    }

    fn add_resource_record(
        &mut self,
        name: String,
        qtype: QueryType,
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

#[derive(Debug, Clone)]
struct Packet {
    header: Header,
    questions: Vec<Question>,
}

impl Packet {
    fn split(&mut self) -> Vec<Vec<u8>> {
        let mut packets: Vec<Vec<u8>> = Vec::new();

        let header = self.header.question_count(47).build();

        for question in &self.questions {
            let mut packet: Vec<u8> = Vec::new();

            packet.extend_from_slice(&header.to_bytes());
            packet.extend_from_slice(&question.to_bytes());

            packets.push(packet);
        }

        packets
    }

    fn merge(&self) -> Vec<u8> {
        let mut response = Vec::new();

        response.extend_from_slice(&self.header.to_bytes());

        for question in self.questions.iter() {
            response.extend_from_slice(&question.to_bytes());
        }

        for rr in self.resource_records.iter() {
            response.extend_from_slice(&rr.to_bytes());
        }

        response
    }

    fn from_bytes(buf: [u8; 512]) -> Packet {
        let header = Header::from_bytes(&buf[..12]);

        let mut questions: Vec<Question> = Vec::new();

        let mut idx = 12;

        for _ in 0..header.question_count {
            let mut end_of_q = idx;
            while buf[end_of_q] != 0 {
                end_of_q += 1;
            }
            // add 4 corressponding to the `QueryType` (2 bytes) and `Class` (2 bytes)
            end_of_q += 4;

            end_of_q += 1;

            let question = Question::from_bytes(&buf, idx);
            questions.push(question);
            idx = end_of_q;
        }

        Packet { header, questions }
    }
}

#[derive(Debug, Clone)]
struct Question {
    name: String,
    qtype: QueryType,
    class: Class,
}

#[derive(Debug, Clone, Copy)]
enum QueryType {
    A = 1,
    NS = 2,
    MD = 3,
    MF = 4,
    CNAME = 5,
    SOA = 6,
    MB = 7,
    MG = 8,
    MR = 9,
    NULL = 10,
    WKS = 11,
    PTR = 12,
    HINFO = 13,
    MINFO = 14,
    MX = 15,
    TXT = 16,
}

impl QueryType {
    fn to_u16(self) -> u16 {
        self as u16
    }

    fn from_u16(value: u16) -> QueryType {
        match value {
            1 => QueryType::A,
            2 => QueryType::NS,
            3 => QueryType::MD,
            4 => QueryType::MF,
            5 => QueryType::CNAME,
            6 => QueryType::SOA,
            7 => QueryType::MB,
            8 => QueryType::MG,
            9 => QueryType::MR,
            10 => QueryType::NULL,
            11 => QueryType::WKS,
            12 => QueryType::PTR,
            13 => QueryType::HINFO,
            14 => QueryType::MINFO,
            15 => QueryType::MX,
            16 => QueryType::TXT,
            _ => panic!("Unknown QueryType"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Class {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

impl Class {
    fn to_u16(self) -> u16 {
        self as u16
    }

    fn from_u16(value: u16) -> Class {
        match value {
            1 => Class::IN,
            2 => Class::CS,
            3 => Class::CH,
            4 => Class::HS,
            _ => panic!("Unknown Class"),
        }
    }
}

impl Question {
    fn new(name: String, qtype: QueryType, class: Class) -> Self {
        Self { name, qtype, class }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // Encode the name
        for label in self.name.split('.') {
            bytes.push(label.len() as u8);
            bytes.extend_from_slice(label.as_bytes());
        }
        // Null byte to terminate the domain name
        bytes.push(0);

        // Encode `qtype` (2 bytes)
        let qtype = self.qtype.to_u16();
        bytes.push((qtype >> 8) as u8);
        bytes.push(qtype as u8);

        // Encode `class` (2 bytes)
        let class = self.class.to_u16();
        bytes.push((class >> 8) as u8);
        bytes.push(class as u8);

        bytes
    }

    fn from_bytes(data: &[u8], start_idx: usize) -> Question {
        let (label, _) = labels_from_bytes(data.clone(), start_idx);

        Question::new(label, QueryType::A, Class::IN)
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

#[derive(Debug, Clone)]
struct ResourceRecord {
    name: String,
    qtype: QueryType,
    class: Class,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

impl ResourceRecord {
    fn new(
        name: String,
        qtype: QueryType,
        class: Class,
        ttl: u32,
        rdlength: u16,
        rdata: Vec<u8>,
    ) -> Self {
        Self {
            name,
            qtype,
            class,
            ttl,
            rdlength,
            rdata,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // Encode the name
        for label in self.name.split('.') {
            bytes.push(label.len() as u8);
            bytes.extend_from_slice(label.as_bytes());
        }
        // Null byte to terminate the domain name
        bytes.push(0);

        // Encode `qtype` (2 bytes)
        let qtype = self.qtype.to_u16();
        bytes.push((qtype >> 8) as u8);
        bytes.push(qtype as u8);

        // Encode `class` (2 bytes)
        let class = self.class.to_u16();
        bytes.push((class >> 8) as u8);
        bytes.push(class as u8);

        // Encode `ttl` (4 bytes)
        bytes.push((self.ttl >> 24) as u8);
        bytes.push((self.ttl >> 16) as u8);
        bytes.push((self.ttl >> 8) as u8);
        bytes.push(self.ttl as u8);

        // Encode `length` (2 bytes)
        bytes.push((self.rdlength >> 8) as u8);
        bytes.push(self.rdlength as u8);

        // Encode `data` (rdlength bytes)
        bytes.extend_from_slice(&self.rdata);

        bytes
    }

    fn from_bytes(buf: &[u8]) -> ResourceRecord {
        let header = Header::from_bytes(&buf[..12]);

        println!("rr {:?}", buf);
        let mut idx = 12;

        for _ in 0..header.answer_count {
            // Decode the name
            let (name, pos) = labels_from_bytes(buf, idx);
            // let mut name = String::new();
            // while buf[idx] != 0 {
            //     let len = buf[idx] as usize;
            //     idx += 1;
            //     name.push_str(std::str::from_utf8(&buf[idx..idx + len]).unwrap());
            //     idx += len;
            //     if buf[idx] != 0 {
            //         name.push('.');
            //     }
            // }
            // idx += 1; // Skip the null byte
            println!("idx {:?}", idx);
            println!("name {:?}", name);

            idx = pos + 1; // Skip the null byte

            // Decode `qtype` (2 bytes)
            let qtype = QueryType::from_u16(u16::from_be_bytes([buf[idx], buf[idx + 1]]));
            idx += 2;
            println!("qtype {:?}", qtype);

            // Decode `class` (2 bytes)
            let class = Class::from_u16(u16::from_be_bytes([buf[idx], buf[idx + 1]]));
            idx += 2;
            println!("class {:?}", class);
        }

        // Decode `ttl` (4 bytes)
        let ttl = u32::from_be_bytes([buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]]);
        idx += 4;
        println!("ttl {:?}", ttl);

        // Decode `rdlength` (2 bytes)
        let rdlength = u16::from_be_bytes([buf[idx], buf[idx + 1]]);
        idx += 2;
        println!("rdlength {:?}", rdlength);

        // Decode `rdata` (rdlength bytes)
        let rdata = buf[idx..idx + rdlength as usize].to_vec();
        println!("rdata {:?}", rdata);

        ResourceRecord {
            name: "jlsadjfal".to_string(),
            qtype: QueryType::A,
            class: Class::IN,
            ttl,
            rdlength,
            rdata,
        }
    }
}
