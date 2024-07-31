#[derive(Debug, Default, Clone, Copy)]
pub struct Header {
    pub id: u16,
    pub query_response: bool,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncated_msg: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: u8,
    pub response_code: u8,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

impl Header {
    pub fn id(&mut self, id: u16) -> &mut Self {
        self.id = id;
        self
    }

    pub fn query_response(&mut self, query_response: bool) -> &mut Self {
        self.query_response = query_response;
        self
    }

    pub fn opcode(&mut self, opcode: u8) -> &mut Self {
        self.opcode = opcode;
        self
    }

    pub fn authoritative_answer(&mut self, authoritative_answer: bool) -> &mut Self {
        self.authoritative_answer = authoritative_answer;
        self
    }

    pub fn truncated_msg(&mut self, truncated_msg: bool) -> &mut Self {
        self.truncated_msg = truncated_msg;
        self
    }

    pub fn recursion_desired(&mut self, recursion_desired: bool) -> &mut Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn recursion_available(&mut self, recursion_available: bool) -> &mut Self {
        self.recursion_available = recursion_available;
        self
    }

    pub fn reserved(&mut self, reserved: u8) -> &mut Self {
        self.reserved = reserved;
        self
    }

    pub fn response_code(&mut self, response_code: u8) -> &mut Self {
        self.response_code = response_code;
        self
    }

    pub fn question_count(&mut self, question_count: u16) -> &mut Self {
        self.question_count = question_count;
        self
    }

    pub fn answer_count(&mut self, answer_count: u16) -> &mut Self {
        self.answer_count = answer_count;
        self
    }

    pub fn authority_count(&mut self, authority_count: u16) -> &mut Self {
        self.authority_count = authority_count;
        self
    }

    pub fn additional_count(&mut self, additional_count: u16) -> &mut Self {
        self.additional_count = additional_count;
        self
    }

    pub fn build(&self) -> Header {
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

    pub fn inc_qcount(&mut self) {
        self.question_count += 1;
    }

    pub fn inc_ancount(&mut self) {
        self.answer_count += 1;
    }

    pub fn to_bytes(&self) -> [u8; 12] {
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

    pub fn from_bytes(data: &[u8]) -> Header {
        // Parse the ID
        let id = u16::from_be_bytes([data[0], data[1]]);

        // Parse the flags (2 bytes)
        let flags = u16::from_be_bytes([data[2], data[3]]);

        let query_response = (flags & 0x8000) != 0;
        let opcode = ((flags & 0x7800) >> 11) as u8;
        let authoritative_answer = (flags & 0x0400) != 0;
        let truncated_msg = (flags & 0x0200) != 0;
        let recursion_desired = (flags & 0x0100) != 0;
        let recursion_available = (flags & 0x0080) != 0;
        let reserved = ((flags & 0x0070) >> 4) as u8;
        let response_code = (flags & 0x000F) as u8;

        // Parse the counts
        let question_count = u16::from_be_bytes([data[4], data[5]]);
        let answer_count = u16::from_be_bytes([data[6], data[7]]);
        let authority_count = u16::from_be_bytes([data[8], data[9]]);
        let additional_count = u16::from_be_bytes([data[10], data[11]]);

        Self {
            id,
            query_response,
            opcode,
            authoritative_answer,
            truncated_msg,
            recursion_desired,
            recursion_available,
            reserved,
            response_code,
            question_count,
            answer_count,
            authority_count,
            additional_count,
        }
    }
}
