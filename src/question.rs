use crate::field::{Class, QType};

#[derive(Debug, Clone)]
pub struct Question {
    pub name: String,
    pub qtype: QType,
    pub class: Class,
}

impl Question {
    pub fn new(name: String, qtype: QType, class: Class) -> Self {
        Self { name, qtype, class }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
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

    pub fn from_bytes(data: &[u8], start_idx: usize) -> Question {
        let (label, _) = labels_from_bytes(data.clone(), start_idx);

        Question::new(label, QType::A, Class::IN)
    }
}

// TODO: move as Label struct method
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
