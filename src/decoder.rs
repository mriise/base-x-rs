#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use bigint::BigUint;
use bigintstatic::BigUintStatic;

use DecodeError;

pub(crate) trait Decoder<'a, 'b>
where
    <Self::Iter as Iterator>::Item: core::cmp::PartialEq + Copy,
{
    type Iter: core::iter::Iterator;

    fn iter(&'a str) -> Self::Iter;
    fn carry(&self, <Self::Iter as core::iter::Iterator>::Item) -> Option<u32>;
    fn alphabet<'c>(&self) -> &'c [<Self::Iter as core::iter::Iterator>::Item]
    where
        'b: 'c;
    #[cfg(feature = "alloc")]
    fn decode(&self, input: &'a str) -> Result<Vec<u8>, DecodeError> {
        if input.is_empty() {
            return Ok(Vec::new());
        }
        let alpha = self.alphabet();
        let base = alpha.len() as u32;

        let mut big = BigUint::with_capacity(4);

        for c in Self::iter(input) {
            if let Some(carry) = self.carry(c) {
                big.mul_add(base, carry);
            } else {
                return Err(DecodeError);
            }
        }

        let mut bytes = big.into_bytes_be();

        let leader = alpha[0];

        let leaders = Self::iter(input).take_while(|byte| *byte == leader).count();

        for _ in 0..leaders {
            bytes.insert(0, 0);
        }

        Ok(bytes)
    }
    /// WARNING: `BACKING` is the size of backing [u32], use const helper fn to find calculate the right backing array size
    fn decode_mut<const BACKING: usize>(
        &self,
        output: &mut [u8],
        input: &'a str,
    ) -> Result<(), DecodeError> {
        if input.is_empty() {
            return Ok(());
        }
        let alpha = self.alphabet();
        let base = alpha.len() as u32;

        let mut big = BigUintStatic::<BACKING>::default();

        for c in Self::iter(input) {
            if let Some(carry) = self.carry(c) {
                match big.mul_add(base, carry) {
                    Ok(_) => (),
                    Err(_) => return Err(DecodeError),
                }
            } else {
                return Err(DecodeError);
            }
        }

        //TODO better error handling
        match big.into_bytes_be(output) {
            Ok(_) => (),
            Err(_) => return Err(DecodeError),
        }

        let leader = alpha[0];

        let leaders = Self::iter(input).take_while(|byte| *byte == leader).count();

        let rotate_by = output.len() - leaders;
        output.rotate_left(rotate_by); // this is O(n), which may need to be optimized

        Ok(())
    }
}

pub(crate) struct U8Decoder<'b> {
    alphabet: &'b [u8],
    lookup: [u8; 256],
}

impl<'a> U8Decoder<'a> {
    #[inline]
    pub(crate) fn new(alphabet: &'a [u8]) -> Self {
        const INVALID_INDEX: u8 = 0xFF;
        let mut lookup = [INVALID_INDEX; 256];

        for (i, byte) in alphabet.iter().enumerate() {
            lookup[*byte as usize] = i as u8;
        }
        U8Decoder { alphabet, lookup }
    }
}

impl<'a, 'b> Decoder<'a, 'b> for U8Decoder<'b> {
    type Iter = core::str::Bytes<'a>;
    #[inline]
    fn iter(s: &'a str) -> Self::Iter {
        s.bytes()
    }
    #[inline]
    fn carry(&self, c: u8) -> Option<u32> {
        match self.lookup[c as usize] {
            0xFF => None,
            index => Some(index.into()),
        }
    }
    #[inline]
    fn alphabet<'c>(&self) -> &'c [u8]
    where
        'b: 'c,
    {
        self.alphabet
    }
}

pub(crate) struct CharDecoder<'b>(pub &'b [char]);

impl<'a, 'b> Decoder<'a, 'b> for CharDecoder<'b> {
    type Iter = core::str::Chars<'a>;

    #[inline]
    fn iter(s: &'a str) -> Self::Iter {
        s.chars()
    }
    #[inline]
    fn carry(&self, c: char) -> Option<u32> {
        self.0
            .iter()
            .enumerate()
            .find(|&(_, ch)| *ch == c)
            .map(|(i, _)| i as u32)
    }
    #[inline]
    fn alphabet<'c>(&self) -> &'c [char]
    where
        'b: 'c,
    {
        self.0
    }
}
