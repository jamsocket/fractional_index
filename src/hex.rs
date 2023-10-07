use std::{error::Error, fmt::Display};

const HEX_CHARS: &[u8] = b"0123456789abcdef";

pub fn byte_to_hex(byte: u8) -> String {
    let mut s = String::new();
    s.push(HEX_CHARS[(byte >> 4) as usize] as char);
    s.push(HEX_CHARS[(byte & 0xf) as usize] as char);
    s
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push_str(&byte_to_hex(*byte));
    }
    s
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, InvalidChar> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in 0..hex.len() / 2 {
        bytes.push(hex_to_byte(&hex[i * 2..i * 2 + 2])?);
    }
    Ok(bytes)
}

#[derive(Debug)]
pub struct InvalidChar(char);

impl Display for InvalidChar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid hex character: {}", self.0)
    }
}

impl Error for InvalidChar {
    fn description(&self) -> &str {
        "Invalid hex character"
    }
}

pub fn hex_to_byte(hex: &str) -> Result<u8, InvalidChar> {
    let mut byte = 0;
    for c in hex.chars() {
        byte <<= 4;
        match c {
            '0'..='9' => byte += c as u8 - b'0',
            'a'..='f' => byte += c as u8 - b'a' + 10,
            _ => return Err(InvalidChar(c)),
        }
    }
    Ok(byte)
}
