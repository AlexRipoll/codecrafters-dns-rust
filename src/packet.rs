#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub resource_records: Vec<ResourceRecord>,
}

impl Packet {
    pub fn split(&mut self) -> Vec<Vec<u8>> {
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

    pub fn merge(&self) -> Vec<u8> {
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

    pub fn from_bytes(buf: [u8; 512]) -> Packet {
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

        Packet {
            header,
            questions,
            resource_records: vec![],
        }
    }
}
