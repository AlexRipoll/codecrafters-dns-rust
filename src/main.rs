use std::net::UdpSocket;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let header = DnsHeader::new();
                let response = header.to_bytes();
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
        bytes[11] = self.additional_count.to_be() as u8;

        bytes
    }
}
