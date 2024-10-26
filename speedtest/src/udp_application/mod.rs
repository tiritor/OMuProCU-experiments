
pub enum UDPApplicationEnum {
    REQUEST = 0,
    RESPONSE = 1,
}

#[derive(Debug)]
pub struct UDPApplication {
    pub type_field: u16,
    pub session_id: u16,
    pub test_payload: Vec<u8>,
}

impl<'a> UDPApplication {

    pub fn new(buf: &[u8]) -> Self {
        let type_field = u16::from_be_bytes([buf[0], buf[1]]);
        let session_id = u16::from_be_bytes([buf[2], buf[3]]);
        let test_payload = buf[4..].to_vec();

        UDPApplication {
            type_field,
            session_id,
            test_payload,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Type: {}, Session ID: {}, Payload Size: {}",
            self.type_field,
            self.session_id,
            self.test_payload.len()
        )
    }

    pub fn from_bytes(buf: &[u8]) -> Self {
        let type_field = u16::from_be_bytes([buf[0], buf[1]]);
        let session_id = u16::from_be_bytes([buf[2], buf[3]]);
        let test_payload = buf[4..].to_vec();

        UDPApplication {
            type_field,
            session_id,
            test_payload,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push((self.type_field >> 8) as u8);
        bytes.push(self.type_field as u8);
        bytes.push((self.session_id >> 8) as u8);
        bytes.push(self.session_id as u8);
        bytes.extend_from_slice(&self.test_payload);
        bytes
    }
}