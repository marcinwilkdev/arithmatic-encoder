use crate::encoder::{self, HALF};

pub fn decode(len: usize, symbols_count: usize, d: &[u8], cumulative: &[u32], m: usize) -> Vec<u8> {
    let mut text = Vec::new();

    let mut b = 0;
    let mut l = u32::MAX;
    let mut v = 0;

    let mut t = if len < 32 { len } else { 32 };

    for i in 0..d.len() {
        if i > 3 {
            break;
        }

        v += (d[i] as u32) << 8 * (3 - i);
    }

    for _ in 0..symbols_count {
        let s = interval_selection(v, &mut b, &mut l, &cumulative, m);

        text.push(s);

        if l <= HALF {
            decoder_renormalization(&mut v, &mut b, &mut l, &mut t, d, len);
        }
    }

    return text;
}

pub fn adaptive_decode(
    len: usize,
    symbols_count: usize,
    d: &[u8],
    cumulative: &mut [u32],
    m: usize,
) -> Vec<u8> {
    let mut text = Vec::new();

    let mut b = 0;
    let mut l = u32::MAX;
    let mut v = 0;

    let mut t = if len < 32 { len - 1 } else { 31 };

    for i in 0..d.len() {
        if i > 3 {
            break;
        }

        v += (d[i] as u32) << 8 * (3 - i);
    }

    for i in 0..symbols_count {
        let cm = i as u64 + 1 + m as u64;
        let s = adaptive_interval_selection(v, &mut b, &mut l, cumulative, cm, m);

        text.push(s);

        if l <= HALF {
            decoder_renormalization(&mut v, &mut b, &mut l, &mut t, d, len);
        }

        encoder::update_distribution(s, cumulative, m);
    }

    return text;
}

fn interval_selection(v: u32, b: &mut u32, l: &mut u32, cumulative: &[u32], m: usize) -> u8 {
    let mut s = m - 1;

    let temp = *l as u64 * cumulative[(m - 1) as usize] as u64;
    let mut x = *b + (temp >> 32) as u32;
    let mut y = *b + *l;

    loop {
        if v >= *b {
            if x >= *b && x <= v {
                break;
            }
        } else {
            if x >= *b {
                if y > v {
                    break;
                }
            } else {
                if x < v {
                    break;
                }
            }
        }

        s -= 1;
        y = x;
        let temp = *l as u64 * cumulative[s as usize] as u64;
        x = *b + (temp >> 32) as u32;
    }

    *b = x;
    *l = y - *b;

    return s.try_into().expect("symbol too large");
}

fn adaptive_interval_selection(
    v: u32,
    b: &mut u32,
    l: &mut u32,
    cumulative: &[u32],
    cm: u64,
    m: usize,
) -> u8 {
    let mut s = m - 1;
    let scale = u32::MAX as u64 / cm;

    let temp = *l as u64 * cumulative[(m - 1) as usize] as u64 * scale;
    let mut x = *b + (temp >> 32) as u32;
    let mut y = *b + *l;

    loop {
        if v >= *b {
            if x >= *b && x <= v {
                break;
            }
        } else {
            if x >= *b {
                if y > v {
                    break;
                }
            } else {
                if x < v {
                    break;
                }
            }
        }

        s -= 1;
        y = x;
        let temp = *l as u64 * cumulative[s as usize] as u64 * scale;
        x = *b + (temp >> 32) as u32;
    }

    *b = x;
    *l = y - *b;

    return s.try_into().expect("symbol too large");
}

fn decoder_renormalization(
    v: &mut u32,
    b: &mut u32,
    l: &mut u32,
    t: &mut usize,
    d: &[u8],
    len: usize,
) {
    while *l <= HALF {
        *b <<= 1;
        *v <<= 1;

        *t += 1;

        if *t < len {
            let byte_index = *t / 8;
            let bit_index = 8 - (*t % 8) - 1;
            let mask = 2_u8.pow(bit_index as u32);

            let mut bit = mask & d[byte_index];
            bit >>= bit_index;

            *v |= bit as u32;
        }

        *l <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_selection_works() {
        let w = u32::MAX / 4;
        let x = w;
        let y = w;
        let z = u32::MAX - w - x - y;

        let cumulative = [0, w, w + x, w + x + y, w + x + y + z];
        let mut b = 0;
        let mut l = u32::MAX;
        let v = w + 10;

        let s = interval_selection(v, &mut b, &mut l, &cumulative, 4);

        assert_eq!(1, s);
        assert_eq!(w, b + 1);
        assert_eq!(x, b + 1);
    }

    #[test]
    fn decoder_renormalization_works() {}

    #[test]
    fn decode_works() {
        let zero = u32::MAX / 5;
        let one = u32::MAX / 2;
        let two = zero;

        let cumulative = [0, zero, zero + one, zero + one + two, u32::MAX];
        let d = [0b10111110, 0b00100000];
        let symbols_count = 6;
        let len = 13;

        let text = decode(len, symbols_count, &d, &cumulative, 4);

        assert_eq!([2, 1, 0, 0, 1, 3], &text[..]);
    }

    #[test]
    fn adaptive_decode_works() {
        let zero = 1;
        let one = 1;
        let two = 1;
        let three = 1;

        let mut cumulative = [
            0,
            zero,
            zero + one,
            zero + one + two,
            zero + one + two + three,
        ];
        let d = [111, 24];
        let symbols_count = 6;
        let len = 15;

        let text = adaptive_decode(len, symbols_count, &d, &mut cumulative, 4);

        assert_eq!([2, 1, 0, 0, 1, 3], &text[..]);
    }

    #[test]
    fn adaptive_decode_256_signs_works() {
        let mut cumulative = [0; 257];

        for i in 0..257 {
            cumulative[i] = i as u32;
        }

        let d = [1, 254, 255, 3, 246, 200];
        let symbols_count = 6;
        let len = 47;

        let text = adaptive_decode(len, symbols_count, &d, &mut cumulative, 256);

        assert_eq!([2, 1, 0, 0, 1, 3], &text[..]);
    }
}
