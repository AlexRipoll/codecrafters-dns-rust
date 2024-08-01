use crate::{
    header::{self, Header},
    question::{self, Question},
    resource_records::{self, Record},
};

#[derive(Debug, Clone)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub records: Vec<Record>,
}

impl Packet {
    pub fn split(&mut self) -> Vec<Packet> {
        let header = self.header.question_count(1).build();

        self.questions
            .clone()
            .into_iter()
            .map(|question| Self {
                header,
                questions: vec![question],
                records: vec![],
            })
            .collect()
    }

    pub fn merge(packets: Vec<Packet>) -> Packet {
        let mut header = packets[0].header;
        let header = header
            .question_count(packets.len() as u16)
            .answer_count(packets.len() as u16)
            .build();

        let mut questions: Vec<Question> = Vec::new();
        let mut records: Vec<Record> = Vec::new();

        packets.into_iter().for_each(|packet| {
            questions.extend(packet.questions);
            records.extend(packet.records);
        });

        Packet {
            header,
            questions,
            records,
        }
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
            records: vec![],
        }
    }
}
