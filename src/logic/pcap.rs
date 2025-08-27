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
    fn new(address: String, bytes: &[u8]) -> Self {
        let hex: String = bytes
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if i < 8 {
                    format!("{:02x} ", b)
                } else {
                    format!(" {:02x}", b)
                }
            })
            .collect();

        let ascii = bytes
            .iter()
            .map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' })
            .collect::<String>();

        PCapLine {
            address,
            hex,
            ascii,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PCap {}

impl PCap {
    pub fn build(data: Vec<u8>) -> String {
        let mut lines = Vec::new();
        let chunks = data.chunks(16);

        chunks.enumerate().for_each(|(i, bytes)| {
            let address = format!("{:08x}", i * 16);
            let line = PCapLine::new(address, bytes);

            lines.push(line);
        });

        Table::new(lines)
            .with(Remove::row(Rows::first()))
            .with(Style::blank())
            .to_string()
    }
}
