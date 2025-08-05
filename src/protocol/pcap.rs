use std::iter::repeat;

use tabled::{
    Table, Tabled,
    settings::{Remove, Style, object::Rows},
};

#[derive(Tabled, Debug, Clone)]
struct PCapLine {
    address: String,
    hex: String,
    ascii: String,
}

impl PCapLine {
    fn new(address: String, hex: Vec<u8>) -> Self {
        let hex_str = hex
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(" ");

        let mut ascii_str = hex
            .iter()
            .map(|b| {
                if *b >= 32 && *b <= 126 {
                    *b as char
                } else {
                    '.'
                }
            })
            .collect::<String>();

        ascii_str.extend(repeat('.').take(16 - hex.len()));

        PCapLine {
            address,
            hex: hex_str,
            ascii: ascii_str,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PCap {}

impl PCap {
    pub fn build(data: Vec<u8>) -> String {
        let mut lines = Vec::new();
        let chunks: Vec<&[u8]> = data.chunks(16).collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let address = format!("{:08x}", i * 16);
            let line = PCapLine::new(address, chunk.to_vec());

            lines.push(line);
        }

        Table::new(lines)
            .with(Remove::row(Rows::first()))
            .with(Style::blank())
            .to_string()
    }
}
