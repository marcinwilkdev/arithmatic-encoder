mod bits;
mod decoder;
mod encoder;
mod statistics;

pub use decoder::adaptive_decode;
pub use encoder::adaptive_encode;
pub use encoder::EncodedData;
pub use statistics::{show_compression_ratio_and_symbol_len, show_file_entropy};

pub fn cast_u64_to_bytes(number: u64) -> Vec<u8> {
    let mut bytes = vec![0; 8];

    for i in 0..8 {
        let shift = (7 - i) * 8;
        bytes[i] = (number >> shift) as u8;
    }

    bytes
}

pub fn cast_bytes_to_u64(bytes: &[u8]) -> u64 {
    assert_eq!(8, bytes.len(), "Bytes slice has to have length of 8");

    let mut number = 0;

    for i in 0..8 {
        let shift = (7 - i) * 8;
        number += (bytes[i] as u64) << shift;
    }

    number
}

pub fn gen_cumulative(alphabet_len: usize) -> Vec<u32> {
    assert!(alphabet_len < u32::MAX as usize, "Alphabet len too big");

    (0..=alphabet_len).map(|s| s as u32).collect()
}
