pub enum BIT {
    ZERO,
    ONE,
}

pub fn propagate_carry(byte: &mut u8, curr_bit: usize) -> bool {
    assert!(curr_bit < 8, "curr_bit has to be less than 8");

    let mut bit_repr = 2_u8.pow(curr_bit as u32) as u8;

    loop {
        if (*byte & bit_repr) > 0 {
            // swap 1 with 0
            *byte -= bit_repr;
        } else {
            // swap 0 with 1
            *byte += bit_repr;
            return false;
        }

        if bit_repr == 128 {
            // propagate carry futher
            return true;
        }

        bit_repr <<= 1;
    }
}

pub fn push_bit_into_byte(byte: &mut u8, curr_bit: usize) {
    assert!(curr_bit < 8, "curr_bit has to be less than 8");

    let bit_repr = 2_u8.pow(curr_bit as u32) as u8;

    *byte |= bit_repr;
}

pub fn push_bit_into_compressed(curr_bit: &mut usize, bit: BIT, compressed: &mut Vec<u8>) {
    if let BIT::ONE = bit {
        let compressed_len = compressed.len();
        push_bit_into_byte(&mut compressed[compressed_len - 1], *curr_bit);
    }
}

pub fn check_last_byte_full(curr_bit: &mut usize, compressed: &mut Vec<u8>) {
    if *curr_bit == 0 {
        compressed.push(0);
        *curr_bit = 8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagate_carry_working() {
        let mut byte = 96;
        let t = 5;

        let carry = propagate_carry(&mut byte, t);

        assert_eq!(128, byte);
        assert!(!carry);

        let mut byte = 14;
        let t = 1;

        let carry = propagate_carry(&mut byte, t);

        assert!(!carry);
    }

    #[test]
    fn propagate_carry_with_carry_working() {
        let mut byte = 192;
        let t = 6;

        let carry = propagate_carry(&mut byte, t);

        assert_eq!(0, byte);
        assert!(carry);
    }

    #[test]
    fn shifting() {
        let mut byte = 128u8;

        byte <<= 1;

        assert_eq!(0, byte);
    }
}
