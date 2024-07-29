use std::net::UdpSocket;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
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

#[derive(Debug, Default, Clone, Copy)]
struct Header {
    id: u16,
    query_response: bool,
    opcode: u8,
    authoritative_answer: bool,
    truncated_msg: bool,
    recursion_desired: bool,
    recursion_available: bool,
    reserved: u8,
    response_code: u8,
    question_count: u16,
    answer_count: u16,
    authority_count: u16,
    additional_count: u16,
}

impl Header {
    fn id(self, id: u16) -> Self {
        Self { id, ..self }
    }

    fn query_response(self, query_response: bool) -> Self {
        Self {
            query_response,
            ..self
        }
    }

    fn opcode(self, opcode: u8) -> Self {
        Self { opcode, ..self }
    }

    fn authoritative_answer(self, authoritative_answer: bool) -> Self {
        Self {
            authoritative_answer,
            ..self
        }
    }

    fn truncated_msg(self, truncated_msg: bool) -> Self {
        Self {
            truncated_msg,
            ..self
        }
    }

    fn recursion_desired(self, recursion_desired: bool) -> Self {
        Self {
            recursion_desired,
            ..self
        }
    }

    fn recursion_available(self, recursion_available: bool) -> Self {
        Self {
            recursion_available,
            ..self
        }
    }

    fn reserved(self, reserved: u8) -> Self {
        Self { reserved, ..self }
    }

    fn response_code(self, response_code: u8) -> Self {
        Self {
            response_code,
            ..self
        }
    }

    fn question_count(self, question_count: u16) -> Self {
        Self {
            question_count,
            ..self
        }
    }

    fn answer_count(self, answer_count: u16) -> Self {
        Self {
            answer_count,
            ..self
        }
    }

    fn authority_count(self, authority_count: u16) -> Self {
        Self {
            authority_count,
            ..self
        }
    }

    fn additional_count(self, additional_count: u16) -> Self {
        Self {
            additional_count,
            ..self
        }
    }

    fn build(self) -> Header {
        Self {
            id: self.id,
            query_response: self.query_response,
            opcode: self.opcode,
            authoritative_answer: self.authoritative_answer,
            truncated_msg: self.truncated_msg,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            reserved: self.reserved,
            response_code: self.response_code,
            question_count: self.question_count,
            answer_count: self.answer_count,
            authority_count: self.authority_count,
            additional_count: self.additional_count,
        }
    }

    fn inc_qcount(&mut self) {
        self.question_count += 1;
    }

    fn inc_ancount(&mut self) {
        self.answer_count += 1;
    }

    fn to_bytes(&self) -> [u8; 12] {
        let mut bytes = [0u8; 12];

        bytes[..2].copy_from_slice(&self.id.to_be_bytes());

        // Serialize flags and opcodes (16 bits in total)
        bytes[2] = ((self.query_response as u8) << 7)
            | ((self.opcode & 0x0F) << 3)
            | ((self.authoritative_answer as u8) << 2)
            | ((self.truncated_msg as u8) << 1)
            | (self.recursion_desired as u8);

        bytes[3] = ((self.recursion_available as u8) << 7)
            | ((self.reserved & 0x07) << 4)
            | (self.response_code & 0x0F);

        // Serialize `question_count` (16 bits)
        bytes[4] = (self.question_count >> 8) as u8;
        bytes[5] = self.question_count as u8;

        // Serialize `answer_count` (16 bits)
        bytes[6] = (self.answer_count >> 8) as u8;
        bytes[7] = self.answer_count as u8;

        // Serialize `authority_count` (16 bits)
        bytes[8] = (self.authority_count >> 8) as u8;
        bytes[9] = self.authority_count as u8;

        // Serialize `additional_count` (16 bits)
        bytes[10] = (self.additional_count >> 8) as u8;
        bytes[11] = self.additional_count as u8;

        bytes
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
