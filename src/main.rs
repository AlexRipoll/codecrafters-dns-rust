use std::net::UdpSocket;

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

                header = header.id(1234).query_response(true).build();

                let question =
                    Question::new("codecrafters.io".to_string(), QueryType::A, Class::IN);

                let mut dns = Dns::new(header, question);
                dns.header.inc_qcount();

                dns.add_resource_record(
                    "codecrafters.io".to_string(),
                    QueryType::A,
                    Class::IN,
                    60,
                    4,
                    vec![8, 8, 8, 8],
                );
                dns.header.inc_ancount();

                let response = dns.response();

                println!("{:?}", response);

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

#[derive(Debug)]
struct Dns {
    header: Header,
    question: Question,
    resource_records: Vec<ResourceRecord>,
}

impl Dns {
    fn new(header: Header, question: Question) -> Self {
        Self {
            header,
            question,
            resource_records: Vec::new(),
        }
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
        // TODO: check rdata length eq to rdlength
        self.resource_records.push(ResourceRecord::new(
            name, qtype, class, ttl, rdlength, rdata,
        ));
    }

    fn response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        response.extend_from_slice(&self.header.to_bytes());
        response.extend_from_slice(&self.question.to_bytes());
        for rr in self.resource_records.iter() {
            response.extend_from_slice(&rr.to_bytes());
        }

        response
    }
}

#[derive(Debug)]
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
}

#[derive(Debug)]
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
