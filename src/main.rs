use std::net::UdpSocket;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let header = DnsHeader::new();
    let question = DnsQuestion::new("codecrafters.io".to_string(), QueryType::A, Class::IN);

    let mut dns = Dns::new(header, question);

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                dns.header.set_qcount(1);
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
    header: DnsHeader,
    question: DnsQuestion,
}

impl Dns {
    fn new(header: DnsHeader, question: DnsQuestion) -> Self {
        Self { header, question }
    }

    fn response(&self) -> Vec<u8> {
        let mut response = Vec::new();

        response.extend_from_slice(&self.header.to_bytes());
        response.extend_from_slice(&self.question.to_bytes());

        response
    }
}

#[derive(Debug)]
struct DnsHeader {
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

impl DnsHeader {
    fn new() -> Self {
        Self {
            id: 1234,
            query_response: true,
            opcode: 0,
            authoritative_answer: false,
            truncated_msg: false,
            recursion_desired: false,
            recursion_available: false,
            reserved: 0,
            response_code: 0,
            question_count: 0,
            answer_count: 0,
            authority_count: 0,
            additional_count: 0,
        }
    }

    fn set_qcount(&mut self, count: u16) {
        self.question_count = count;
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
struct DnsQuestion {
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

impl DnsQuestion {
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
