use crate::bits;
use crate::bits::BIT;
use crate::helpers::{self, ALPHABET_LEN};

pub const HALF: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
pub const BLOCK_SIZE: usize = 128;

#[derive(Debug)]
pub struct EncodedData {
    pub len: usize,
    pub data: Vec<u8>,
}

pub fn compress(data: &[u8], alphabet_len: usize) -> EncodedData {
    let mut compressed = vec![0];
    let mut cumulative_distribution = helpers::gen_cumulative();

    // interval struct
    let mut begin_interval = 0; // b
    let mut len_interval = u32::MAX; // l
    let mut curr_bit = 8; // t

    let mut internal_block_index = 0;
    let mut occurences = [1; ALPHABET_LEN];

    for (letter_index, letter) in data.iter().enumerate() {
        let begin_interval_copy = begin_interval;
        let cumulative_distribution_sum = letter_index as u64 + alphabet_len as u64 + 1;

        // 8ms
        adaptive_interval_update(
            *letter,
            &mut begin_interval,
            &mut len_interval,
            &cumulative_distribution,
            cumulative_distribution_sum,
            alphabet_len,
        );

        if begin_interval < begin_interval_copy {
            propagate_carry(curr_bit, &mut compressed);
        }

        // 8ms
        if len_interval <= HALF {
            encoder_renormalization(
                &mut begin_interval,
                &mut len_interval,
                &mut curr_bit,
                &mut compressed,
            );
        }

        update_block(&mut occurences, &mut internal_block_index, *letter);

        if internal_block_index == 0 {
            update_distribution_block(&occurences, &mut cumulative_distribution);
        }
    }

    code_value_selection(begin_interval, &mut curr_bit, &mut compressed);

    EncodedData {
        len: compressed.len() * 8 - curr_bit,
        data: compressed,
    }
}

fn adaptive_interval_update(
    letter: u8,
    begin_interval: &mut u32,
    len_interval: &mut u32,
    cumulative_distribution: &[u32],
    cumulative_distribution_sum: u64,
    alphabet_len: usize,
) {
    let scale = u32::MAX as u64 / cumulative_distribution_sum;

    let y = if letter as usize == alphabet_len - 1 {
        *begin_interval + *len_interval
    } else {
        let temp =
            *len_interval as u64 * cumulative_distribution[letter as usize + 1] as u64 * scale;
        *begin_interval + (temp >> 32) as u32
    };

    let temp = *len_interval as u64 * cumulative_distribution[letter as usize] as u64 * scale;

    *begin_interval += (temp >> 32) as u32;
    *len_interval = y - *begin_interval;
}

fn propagate_carry(mut curr_bit: usize, compressed: &mut [u8]) {
    for i in (0..compressed.len()).rev() {
        match bits::propagate_carry(&mut compressed[i], curr_bit) {
            true => curr_bit = 0,
            false => break,
        }
    }
}

fn encoder_renormalization(
    begin_interval: &mut u32,
    len_interval: &mut u32,
    curr_bit: &mut usize,
    compressed: &mut Vec<u8>,
) {
    while *len_interval < HALF {
        bits::check_last_byte_full(curr_bit, compressed);

        *curr_bit -= 1;
        *len_interval <<= 1;

        if *begin_interval >= HALF {
            bits::push_bit_into_compressed(curr_bit, BIT::ONE, compressed);
        } else {
            bits::push_bit_into_compressed(curr_bit, BIT::ZERO, compressed);
        }

        *begin_interval <<= 1;
    }
}

fn code_value_selection(begin_interval: u32, curr_bit: &mut usize, compressed: &mut Vec<u8>) {
    if begin_interval < HALF {
        bits::check_last_byte_full(curr_bit, compressed);

        *curr_bit -= 1;

        bits::push_bit_into_compressed(curr_bit, BIT::ONE, compressed);
    } else {
        propagate_carry(*curr_bit, compressed);

        bits::check_last_byte_full(curr_bit, compressed);

        *curr_bit -= 1;
    }
}

// no test
pub fn update_block(
    occurences: &mut [u32; ALPHABET_LEN],
    internal_block_index: &mut usize,
    letter: u8,
) {
    occurences[letter as usize] += 1;
    *internal_block_index += 1;

    if *internal_block_index > BLOCK_SIZE - 1 {
        *internal_block_index = 0;
    }
}

// no test
pub fn update_distribution_block(
    occurences: &[u32; ALPHABET_LEN],
    cumulative_distribution: &mut [u32],
) {
    let mut total = 0;

    for i in 0..occurences.len() {
        total += occurences[i];
        cumulative_distribution[i + 1] = total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagate_carry_works() {
        let mut d = [128, 14];
        let t = 1;

        propagate_carry(t, &mut d);

        assert_eq!([128, 16], d);

        let mut d = [128, 192];
        let t = 6;

        propagate_carry(t, &mut d);

        assert_eq!([129, 0], d);
    }

    #[test]
    fn encoder_renormalization_works() {
        let mut b = 0b0111_1111_1111_1111_1111_1111_1111_1111;
        let mut l = 0b0011_1111_1111_1111_1111_1111_1111_1111;

        let mut d = vec![128];
        let mut t = 1;

        encoder_renormalization(&mut b, &mut l, &mut t, &mut d);

        assert_eq!(0b11_1111_1111_1111_1111_1111_1111_1111_00, b);
        assert_eq!(0b11_1111_1111_1111_1111_1111_1111_1111_00, l);
        assert_eq!(vec![128, 128], d);
    }

    #[test]
    fn code_value_selection_works() {
        let b = HALF;
        let mut d = vec![129];
        let mut t = 0;

        code_value_selection(b, &mut t, &mut d);

        assert_eq!(vec![130, 0], d);
    }

    #[test]
    fn adaptive_encode_works() {
        let data = [2, 1, 0, 0, 1, 3];

        let EncodedData { data, len } = compress(&data, 4);

        assert_eq!(111, data[0]);
        assert_eq!(24, data[1]);
        assert_eq!(15, len);
    }

    #[test]
    fn adaptive_encode_256_signs_works() {
        let data = [2, 1, 0, 0, 1, 3];

        let EncodedData { data, len } = compress(&data, 256);

        assert_eq!(1, data[0]);
        assert_eq!(254, data[1]);
        assert_eq!(47, len);
    }
}
