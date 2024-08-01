use crate::{
    field::{Class, QType},
    header::Header,
};

#[derive(Debug, Clone)]
pub struct ResourceRecord {
    pub name: String,
    pub qtype: QType,
    pub class: Class,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    pub fn new(
        name: String,
        qtype: QType,
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

    pub fn from_bytes(buf: &[u8]) -> ResourceRecord {
        let header = Header::from_bytes(&buf[..12]);

        println!("rr {:?}", buf);
        let mut idx = 12;

        for _ in 0..header.answer_count {
            // Decode the name
            let (name, pos) = labels_from_bytes(buf, idx);
            // let mut name = String::new();
            // while buf[idx] != 0 {
            //     let len = buf[idx] as usize;
            //     idx += 1;
            //     name.push_str(std::str::from_utf8(&buf[idx..idx + len]).unwrap());
            //     idx += len;
            //     if buf[idx] != 0 {
            //         name.push('.');
            //     }
            // }
            // idx += 1; // Skip the null byte
            println!("idx {:?}", idx);
            println!("name {:?}", name);

            idx = pos + 1; // Skip the null byte

            // Decode `qtype` (2 bytes)
            let qtype = QType::from_u16(u16::from_be_bytes([buf[idx], buf[idx + 1]]));
            idx += 2;
            println!("qtype {:?}", qtype);

            // Decode `class` (2 bytes)
            let class = Class::from_u16(u16::from_be_bytes([buf[idx], buf[idx + 1]]));
            idx += 2;
            println!("class {:?}", class);
        }

        // Decode `ttl` (4 bytes)
        let ttl = u32::from_be_bytes([buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]]);
        idx += 4;
        println!("ttl {:?}", ttl);

        // Decode `rdlength` (2 bytes)
        let rdlength = u16::from_be_bytes([buf[idx], buf[idx + 1]]);
        idx += 2;
        println!("rdlength {:?}", rdlength);

        // Decode `rdata` (rdlength bytes)
        let rdata = buf[idx..idx + rdlength as usize].to_vec();
        println!("rdata {:?}", rdata);

        ResourceRecord {
            name: "jlsadjfal".to_string(),
            qtype: QType::A,
            class: Class::IN,
            ttl,
            rdlength,
            rdata,
        }
    }
}

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
