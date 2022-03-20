use crate::bits;

pub const HALF: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;

#[derive(Debug)]
pub struct Encoded {
    pub len: usize,
    pub data: Vec<u8>,
}

const P: usize = 32; // num bits in register

pub fn encode(data: &[u8], cumulative: &[u32], m: usize) -> Encoded {
    let mut d = vec![0];

    let mut b = 0;
    let mut l = u32::MAX;
    let mut t = 8;

    for letter in data {
        let b_copy = b;
        interval_update(*letter, &mut b, &mut l, &cumulative, m);
        // b > 1 -> b overflow
        if b < b_copy {
            propagate_carry(t, &mut d);
        }

        if l <= HALF {
            encoder_renormalization(&mut b, &mut l, &mut t, &mut d);
        }
    }

    code_value_selection(b, &mut t, &mut d);

    Encoded {
        len: ((d.len() - 1) * 8) + 8 - t,
        data: d,
    }
}

pub fn adaptive_encode(data: &[u8], cumulative: &mut [u32], m: usize) -> Encoded {
    let mut d = vec![0];

    let mut b = 0;
    let mut l = u32::MAX;
    let mut t = 8;

    for (k, letter) in data.iter().enumerate() {
        let b_copy = b;
        let cm = k as u64 + 1 + m as u64;
        adaptive_interval_update(*letter, &mut b, &mut l, &cumulative, cm, m);

        if b < b_copy {
            propagate_carry(t, &mut d);
        }

        if l <= HALF {
            encoder_renormalization(&mut b, &mut l, &mut t, &mut d);
        }

        update_distribution(*letter, cumulative, m);
    }

    code_value_selection(b, &mut t, &mut d);

    Encoded {
        len: ((d.len() - 1) * 8) + 8 - t,
        data: d,
    }
}

fn interval_update(letter: u8, b: &mut u32, l: &mut u32, cumulative: &[u32], m: usize) {
    let y = if letter as usize == m - 1 {
        *b + *l
    } else {
        let temp = *l as u64 * cumulative[letter as usize + 1] as u64;
        *b + (temp >> 32) as u32
    };

    let temp = *l as u64 * cumulative[letter as usize] as u64;

    *b += (temp >> 32) as u32;
    *l = y - *b;
}

fn adaptive_interval_update(
    letter: u8,
    b: &mut u32,
    l: &mut u32,
    cumulative: &[u32],
    cm: u64,
    m: usize,
) {
    let scale = u32::MAX as u64 / cm;
    let y = if letter as usize == m - 1 {
        *b + *l
    } else {
        let temp = *l as u64 * cumulative[letter as usize + 1] as u64 * scale;
        *b + (temp >> 32) as u32
    };

    let temp = *l as u64 * cumulative[letter as usize] as u64 * scale;

    *b += (temp >> 32) as u32;
    *l = y - *b;
}

fn propagate_carry(mut t: usize, d: &mut [u8]) {
    for i in (0..d.len()).rev() {
        match bits::propagate_carry(&mut d[i], t) {
            true => t = 0,
            false => break,
        }
    }
}

fn encoder_renormalization(b: &mut u32, l: &mut u32, t: &mut usize, d: &mut Vec<u8>) {
    while *l <= HALF {
        if *t == 0 {
            d.push(0);
            *t = 8;
        }

        *t -= 1;
        *l <<= 1;

        if *b >= HALF {
            push_into_d(t, BIT::ONE, d);
        } else {
            push_into_d(t, BIT::ZERO, d);
        }

        *b <<= 1;
    }
}

enum BIT {
    ZERO,
    ONE,
}

fn push_into_d(t: &mut usize, bit: BIT, d: &mut Vec<u8>) {
    if let BIT::ONE = bit {
        let d_len = d.len();
        bits::push_bit(&mut d[d_len - 1], *t);
    }
}

fn code_value_selection(b: u32, t: &mut usize, d: &mut Vec<u8>) {
    if b < HALF {
        if *t == 0 {
            d.push(0);
            *t = 7;
        } else {
            *t -= 1;
        }

        push_into_d(t, BIT::ONE, d);
    } else {
        propagate_carry(*t, d);

        if *t == 0 {
            d.push(0);
            *t = 7;
        } else {
            *t -= 1;
        }
    }
}

pub fn update_distribution(symbol: u8, cumulative: &mut [u32], m: usize) {
    for m in (symbol as usize) + 1..=m {
        cumulative[m] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_update_works() {
        let w = u32::MAX / 4;
        let x = w;
        let y = w;
        let z = u32::MAX - w - x - y;

        let cumulative = [0, w, w + x, w + x + y, w + x + y + z];
        let mut b = 0;
        let mut l = u32::MAX;
        let letter = 2;

        assert_eq!(u32::MAX, w + x + y + z);

        interval_update(letter, &mut b, &mut l, &cumulative, 4);

        assert_eq!(w + x, b + 1);
        assert_eq!(y, l);
    }

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
    fn encode_works() {
        let zero = u32::MAX / 5;
        let one = u32::MAX / 2;
        let two = zero;

        let cumulative = [0, zero, zero + one, zero + one + two, u32::MAX];

        let data = [2, 1, 0, 0, 1, 3];

        let Encoded { data, len } = encode(&data, &cumulative, 4);

        assert_eq!(0b10111110, data[0]);
        assert_eq!(0b00100000, data[1]);
        assert_eq!(13, len);
    }

    #[test]
    fn update_distribution_works() {
        let mut cumulative = [1; 257];
        let symbol = 1;

        update_distribution(symbol, &mut cumulative, 256);

        assert_eq!([2; 255], &cumulative[2..]);
    }

    #[test]
    fn adaptive_encode_works() {
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

        let data = [2, 1, 0, 0, 1, 3];

        let Encoded { data, len } = adaptive_encode(&data, &mut cumulative, 4);

        assert_eq!(111, data[0]);
        assert_eq!(24, data[1]);
        assert_eq!(15, len);
    }

    #[test]
    fn adaptive_encode_256_signs_works() {
        let mut cumulative = [0; 257];

        for i in 0..257 {
            cumulative[i] = i as u32;
        }

        let data = [2, 1, 0, 0, 1, 3];

        let Encoded { data, len } = adaptive_encode(&data, &mut cumulative, 256);

        assert_eq!(1, data[0]);
        assert_eq!(254, data[1]);
        assert_eq!(47, len);
    }
}
