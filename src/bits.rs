use std::ops::{BitXor, Shl};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bits<const N: usize> {
    inner: u64,
}

impl<const N: usize> Bits<N> {
    pub fn new(inner: u64) -> Bits<N> {
        assert!(64 - inner.leading_zeros() as usize <= N);

        Bits { inner }
    }

    pub fn as_u64(self) -> u64 {
        self.inner
    }

    /// Returns bits within the range, including the lower and upper bound.
    /// GAH! I can't use this because reasons
    pub fn const_range<const FROM: usize, const TO: usize>(self) -> Bits<{ TO - FROM + 1 }> {
        assert!(TO <= N);
        assert!(FROM > 0);
        assert!(FROM < TO);

        Bits {
            inner: (self.inner & (u64::MAX >> (64 - N + FROM - 1))) >> (N - TO),
        }
    }

    pub fn range<const LEN: usize>(self, from: usize, to: usize) -> Bits<LEN> {
        assert!(to <= N);
        assert!(from > 0);
        assert!(from < to);
        assert_eq!(to - from + 1, LEN);

        Bits::new((self.inner & (u64::MAX >> (64 - N + from - 1))) >> (N - to))
    }

    pub fn len(self) -> usize {
        N
    }

    /// Split into left and right
    /// Left value is Msb
    pub fn split<const M: usize>(self) -> (Bits<{ M }>, Bits<{ N - M }>) {
        let lhs = self.inner >> (N / 2);
        let rhs = self.inner & (u64::MAX >> (64 - N / 2));
        (Bits { inner: lhs as u64 }, Bits { inner: rhs as u64 })
    }

    pub fn concat<const M: usize>(self, other: Bits<M>) -> Bits<{ N + M }> {
        assert!(M + N <= 64, "Resulting bit array too long.");

        Bits {
            inner: self.inner << M | other.inner,
        }
    }

    /// Left-indexed, Msb and 1-indexed get function.
    /// If we say (0b0111).get(1) it would get 0.
    pub fn get(self, i: usize) -> bool {
        assert!(i > 0);
        assert!(i <= N);

        ((self.inner >> (N - i)) & 1) == 1
    }

    /// Left-indexed, Msb and 1-indexed get function.
    /// See `get`
    pub fn set(&mut self, i: usize, val: bool) {
        assert!(i > 0);
        assert!(i <= N);

        if val {
            self.inner |= 1 << (N - i);
        } else {
            self.inner &= !(1 << (N - i));
        }
    }

    pub fn permute<const M: usize>(self, permutation: &[u8; M]) -> Bits<M> {
        let mut output = Bits::<M>::new(0);

        // input[0] is the leftmost bit in our code. However in the theory,
        // input[1] is the leftmost bit. So we simply have to substract indices
        // that stem from the theory by 1.

        for (i, j) in permutation.iter().enumerate() {
            // i is 0-indexed, but we operate as 1 indexed
            let i = i + 1;

            // dbg!(i, j);
            output.set(i, self.get(*j as usize));
        }

        output
    }

    pub fn rotate_left(self, rhs: usize) -> Self {
        // We rotate the bits, and then mask off the uneeded high bits
        let rotated = (self.inner << rhs) | (self.inner >> (N - rhs));
        Bits {
            inner: rotated & (u64::MAX >> (64 - N)),
        }
    }
}

impl<const N: usize> BitXor for Bits<N> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bits {
            inner: self.inner ^ rhs.inner,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{IP, IP_INVERSE};

    #[test]
    fn ip_sanity() {
        for (i, j) in IP.iter().enumerate() {
            assert_eq!(IP_INVERSE[(64 - j) as usize], 64 - i as u8);
        }
    }

    #[test]
    fn ip_tranform() {
        let mut input = Bits::<64>::new(0);
        // Set bit 26 to 1, which should be the 5th bit after the transform.
        input.set(26, true);

        let output = input.permute(&IP);
        println!("{:?}", input);
        println!("{:?}", output);

        // Check tht only bit 26 is set
        assert_eq!(output.get(5), true);
        assert_eq!(output.inner.count_ones(), 1);

        // Check that applying the inverse works
        assert_eq!(output.permute(&IP_INVERSE), input);
    }

    #[test]
    fn test_split_bits() {
        let bits: Bits<64> = Bits::new(0xdeadbeefcafebabe);
        let (deadbeef, cafebabe) = bits.split::<32>();

        assert_eq!(deadbeef.inner, 0xdeadbeef);
        assert_eq!(cafebabe.inner, 0xcafebabe);

        let (dead, beef) = deadbeef.split::<16>();
        let (cafe, babe) = cafebabe.split::<16>();

        assert_eq!(dead.inner, 0xdead);
        assert_eq!(beef.inner, 0xbeef);
        assert_eq!(cafe.inner, 0xcafe);
        assert_eq!(babe.inner, 0xbabe);
    }

    #[test]
    fn test_concat() {
        let tatoo: Bits<20> = Bits::new(0x7a700);
        let bae: Bits<12> = Bits::new(0xbae);

        let tatoo_bae = tatoo.concat(bae);
        println!("{} bits: {:x}", tatoo_bae.len(), tatoo_bae.inner);

        assert_eq!(0x7a700bae, tatoo_bae.inner);
    }

    #[test]
    #[should_panic]
    fn test_concat_too_long() {
        let tatoo: Bits<40> = Bits::new(0x7a700);
        let babe: Bits<32> = Bits::new(0xbabe);

        tatoo.concat(babe);
    }

    #[test]
    #[should_panic]
    fn test_valie_too_big() {
        let too_big: Bits<8> = Bits::new(0x700b19);

        println!("This value should panic {:?}!", too_big);
    }

    #[test]
    fn test_get() {
        let one_one: Bits<1> = Bits::new(0b1);
        assert_eq!(one_one.get(1), true);

        let one_one: Bits<2> = Bits::new(0b01);
        assert_eq!(one_one.get(2), true);

        let one_one: Bits<8> = Bits::new(0b0000_1000);
        assert_eq!(one_one.get(5), true);
    }

    #[test]
    fn test_set() {
        let mut one_one: Bits<8> = Bits::new(0b0000_0000);
        one_one.set(5, true);

        assert_eq!(one_one.inner, 0b0000_1000);
    }

    #[test]
    fn test_range() {
        let val: Bits<8> = Bits::new(0b0011_1100);

        assert_eq!(val.const_range::<3, 6>().as_u64(), 0b1111);
        assert_eq!(val.const_range::<3, 6>().len(), 4);

        assert_eq!(val.const_range::<1, 2>().as_u64(), 0b00);
        assert_eq!(val.const_range::<1, 2>().len(), 2);

        assert_eq!(val.const_range::<7, 8>().as_u64(), 0b00);
        assert_eq!(val.const_range::<7, 8>().len(), 2);
    }

    #[test]
    fn test_rotate() {
        let val: Bits<8> = Bits::new(0b0011_1100);

        assert_eq!(val.rotate_left(2).as_u64(), 0b1111_0000);
        assert_eq!(val.rotate_left(4).as_u64(), 0b1100_0011);
    }
}

// I want 1 indexing
// left to right indexing (Msb)
// constant generics (so we can have constant length bitarrays)

// methods
// u64.to_bits::<bit_width>() -> bits
// bits.split() -> (u64, u64)
// bits.permute(permuatation) -> bits
