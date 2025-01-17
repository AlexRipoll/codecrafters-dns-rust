#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QType {
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

impl QType {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn from_u16(value: u16) -> QType {
        match value {
            1 => QType::A,
            2 => QType::NS,
            3 => QType::MD,
            4 => QType::MF,
            5 => QType::CNAME,
            6 => QType::SOA,
            7 => QType::MB,
            8 => QType::MG,
            9 => QType::MR,
            10 => QType::NULL,
            11 => QType::WKS,
            12 => QType::PTR,
            13 => QType::HINFO,
            14 => QType::MINFO,
            15 => QType::MX,
            16 => QType::TXT,
            _ => panic!("Unknown QType"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

impl Class {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn from_u16(value: u16) -> Class {
        match value {
            1 => Class::IN,
            2 => Class::CS,
            3 => Class::CH,
            4 => Class::HS,
            _ => panic!("Unknown Class"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_type_to_u16() {
        assert_eq!(QType::A.to_u16(), 1u16);
        assert_eq!(QType::NS.to_u16(), 2u16);
        assert_eq!(QType::CNAME.to_u16(), 5u16);
    }

    #[test]
    fn test_query_type_from_u16() {
        assert_eq!(QType::from_u16(1), QType::A);
        assert_eq!(QType::from_u16(2), QType::NS);
        assert_eq!(QType::from_u16(5), QType::CNAME);
    }

    #[test]
    fn test_class_to_u16() {
        assert_eq!(Class::IN.to_u16(), 1u16);
        assert_eq!(Class::CS.to_u16(), 2u16);
    }

    #[test]
    fn test_class_from_u16() {
        assert_eq!(Class::from_u16(1), Class::IN);
        assert_eq!(Class::from_u16(2), Class::CS);
    }
}
