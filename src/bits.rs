pub enum BIT {
    ZERO,
    ONE,
}

pub fn propagate_carry(byte: &mut u8, t: usize) -> bool {
    assert!(t < 8, "t has to be less than 8");

    let mut mask = 2_u8.pow(t as u32) as u8;

    loop {
        if (*byte & mask) > 0 {
            // swap 1 with 0
            *byte -= mask;
        } else {
            // swap 0 with 1
            *byte += mask;
            return false;
        }

        if mask == 128 {
            // propagate carry futher
            return true;
        }

        mask <<= 1;
    }
}

pub fn push_bit(byte: &mut u8, t: usize) {
    assert!(t < 8, "t has to be less than 8");

    let mask = 2_u8.pow(t as u32) as u8;

    *byte |= mask;
}

pub fn push_into_d(t: &mut usize, bit: BIT, d: &mut Vec<u8>) {
    if let BIT::ONE = bit {
        let d_len = d.len();
        push_bit(&mut d[d_len - 1], *t);
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
