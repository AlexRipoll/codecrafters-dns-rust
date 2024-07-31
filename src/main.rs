use std::{net::UdpSocket, usize};

use header::Header;

mod header;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    let mut header = Header::default();

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let packet = Packet::from_bytes(buf);

                header = header
                    .id(packet.header.id)
                    .query_response(true)
                    .opcode(packet.header.opcode)
                    .recursion_desired(packet.header.recursion_desired)
                    .response_code(if packet.header.opcode == 0 { 0 } else { 4 })
                    .question_count(packet.header.question_count)
                    .build();

                let mut dns = Dns::new(header.build());
                dns.questions = packet.questions.clone();

                for i in 0..packet.header.question_count {
                    let question = packet.questions.get(i as usize).unwrap();
                    dns.add_resource_record(
                        question.name.clone(),
                        question.qtype,
                        question.class,
                        60,
                        4,
                        vec![8, 8, 8, 8],
                    );
                    dns.header.inc_ancount();
                }

                let response = dns.response();

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

    fn response(&self) -> Vec<u8> {
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
}

#[derive(Debug, Clone)]
struct Packet {
    header: Header,
    questions: Vec<Question>,
}

impl Packet {
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

    fn labels_from_bytes(data: &[u8], start_idx: usize) -> (String, usize) {
        let mut idx = start_idx;
        let mut labels: Vec<String> = Vec::new();

        while data[idx] != 0 {
            if data[idx] & 0b11000000 == 0b11000000 {
                let offset =
                    (((data[idx] & 0b00111111) as u16) << 8 | data[idx + 1] as u16) as usize;
                let (label, _) = Self::labels_from_bytes(&data, offset);
                labels.push(label);
                // increase the index by tw0 since the offset used 2 bytes
                idx += 2;
            } else {
                let length = data[idx] as usize;
                idx += 1;
                let label = std::str::from_utf8(&data[idx..idx + length]).unwrap();
                labels.push(label.to_string());
                idx += length;
            }
        }

        (labels.join("."), idx)
    }

    fn from_bytes(data: &[u8], start_idx: usize) -> Question {
        let (label, _) = Self::labels_from_bytes(data.clone(), start_idx);

        Question::new(label, QueryType::A, Class::IN)
    }
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
}
