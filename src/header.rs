#[derive(Debug, Default, Clone, Copy)]
pub struct Header {
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
    pub fn id(self, id: u16) -> Self {
        Self { id, ..self }
    }

    pub fn query_response(self, query_response: bool) -> Self {
        Self {
            query_response,
            ..self
        }
    }

    pub fn opcode(self, opcode: u8) -> Self {
        Self { opcode, ..self }
    }

    pub fn authoritative_answer(self, authoritative_answer: bool) -> Self {
        Self {
            authoritative_answer,
            ..self
        }
    }

    pub fn truncated_msg(self, truncated_msg: bool) -> Self {
        Self {
            truncated_msg,
            ..self
        }
    }

    pub fn recursion_desired(self, recursion_desired: bool) -> Self {
        Self {
            recursion_desired,
            ..self
        }
    }

    pub fn recursion_available(self, recursion_available: bool) -> Self {
        Self {
            recursion_available,
            ..self
        }
    }

    pub fn reserved(self, reserved: u8) -> Self {
        Self { reserved, ..self }
    }

    pub fn response_code(self, response_code: u8) -> Self {
        Self {
            response_code,
            ..self
        }
    }

    pub fn question_count(self, question_count: u16) -> Self {
        Self {
            question_count,
            ..self
        }
    }

    pub fn answer_count(self, answer_count: u16) -> Self {
        Self {
            answer_count,
            ..self
        }
    }

    pub fn authority_count(self, authority_count: u16) -> Self {
        Self {
            authority_count,
            ..self
        }
    }

    pub fn additional_count(self, additional_count: u16) -> Self {
        Self {
            additional_count,
            ..self
        }
    }

    pub fn build(self) -> Header {
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
}
