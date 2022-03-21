use crate::compressor::{self, HALF};
use crate::helpers;

pub fn decompress(
    bits_count: usize,
    letters_count: usize,
    data: &[u8],
    alphabet_len: usize,
) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut cumulative = helpers::gen_cumulative(alphabet_len);

    let mut begin_interval = 0;
    let mut len_interval = u32::MAX;
    let mut data_chunk = helpers::get_first_data_chunk(data);

    let mut curr_bit = if bits_count < 32 { bits_count - 1 } else { 31 };

    for letter_index in 0..letters_count {
        let cumulative_distribution_sum = letter_index as u64 + alphabet_len as u64 + 1;

        let letter = interval_selection(
            data_chunk,
            &mut begin_interval,
            &mut len_interval,
            &cumulative,
            cumulative_distribution_sum,
            alphabet_len,
        );

        decompressed.push(letter);

        if len_interval <= HALF {
            decoder_renormalization(
                &mut data_chunk,
                &mut begin_interval,
                &mut len_interval,
                &mut curr_bit,
                data,
                bits_count,
            );
        }

        compressor::update_distribution(letter, &mut cumulative, alphabet_len);
    }

    return decompressed;
}

fn interval_selection(
    data_chunk: u32,
    begin_interval: &mut u32,
    len_interval: &mut u32,
    cumulative: &[u32],
    cumulative_distribiution_sum: u64,
    alphabet_len: usize,
) -> u8 {
    let mut letter = alphabet_len - 1;
    let scale = u32::MAX as u64 / cumulative_distribiution_sum;

    let temp = *len_interval as u64 * cumulative[(alphabet_len - 1) as usize] as u64 * scale;
    let mut temp_begin_interval = *begin_interval + (temp >> 32) as u32;
    let mut temp_end_interval = *begin_interval + *len_interval;

    loop {
        if data_chunk >= *begin_interval {
            if temp_begin_interval >= *begin_interval && temp_begin_interval <= data_chunk {
                break;
            }
        } else {
            if temp_begin_interval >= *begin_interval {
                if temp_end_interval > data_chunk {
                    break;
                }
            } else {
                if temp_begin_interval < data_chunk {
                    break;
                }
            }
        }

        letter -= 1;
        temp_end_interval = temp_begin_interval;

        let temp = *len_interval as u64 * cumulative[letter as usize] as u64 * scale;
        temp_begin_interval = *begin_interval + (temp >> 32) as u32;
    }

    *begin_interval = temp_begin_interval;
    *len_interval = temp_end_interval - *begin_interval;

    return letter.try_into().expect("letter too large");
}

fn decoder_renormalization(
    data_chunk: &mut u32,
    begin_interval: &mut u32,
    len_interval: &mut u32,
    curr_bit: &mut usize,
    data: &[u8],
    bits_count: usize,
) {
    while *len_interval <= HALF {
        *begin_interval <<= 1;
        *data_chunk <<= 1;

        *curr_bit += 1;

        if *curr_bit < bits_count {
            let byte_index = *curr_bit / 8;
            let bit_index = 7 - (*curr_bit % 8);
            let bit_repr = 2_u8.pow(bit_index as u32);

            let mut bit = bit_repr & data[byte_index];
            bit >>= bit_index;

            *data_chunk |= bit as u32;
        }

        *len_interval <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_decode_works() {
        let d = [111, 24];
        let symbols_count = 6;
        let len = 15;

        let text = decompress(len, symbols_count, &d, 4);

        assert_eq!([2, 1, 0, 0, 1, 3], &text[..]);
    }

    #[test]
    fn adaptive_decode_256_signs_works() {
        let d = [1, 254, 255, 3, 246, 200];
        let symbols_count = 6;
        let len = 47;

        let text = decompress(len, symbols_count, &d, 256);

        assert_eq!([2, 1, 0, 0, 1, 3], &text[..]);
    }
}
