use core::ptr::copy_nonoverlapping;
/// This is a pretty naive implementation of a BigUint abstracting all
/// math out to an array of `u32` chunks.
///
/// This doesn't define any size constraints to the backing array
///
/// It can only do a few things:
/// - Be instantiated from a big-endian byte slice
/// - Be written to a slice of big-endian bytes.
/// - Do a division by `u32`, mutating self and returning the remainder.
/// - Do a multiplication with addition in one pass.
/// - Check if it's zero.
///
/// Turns out those are all the operations you need to encode and decode
/// base58, or anything else, really.
#[derive(Clone, Copy, Debug)]
pub struct BigUintStatic<const N: usize> {
    chunks: [u32; N],
    len: usize,
}

impl<const N: usize> Default for BigUintStatic<N> {
    fn default() -> Self {
        Self {
            chunks: [0u32; N],
            len: 0,
        }
    }
}

impl<const N: usize> BigUintStatic<N> {
    /// Divide self by `divider`, return the remainder of the operation.
    #[inline]
    pub fn div_mod(&mut self, divider: u32) -> u32 {
        let mut carry = 0u64;

        for chunk in self.chunks.iter_mut() {
            carry = (carry << 32) | u64::from(*chunk);
            *chunk = (carry / u64::from(divider)) as u32;
            carry %= u64::from(divider);
        }

        carry as u32
    }

    /// Perform a multiplication followed by addition. This is a reverse
    /// of `div_mod` in the sense that when supplied remained for addition
    /// and the same base for multiplication as divison, the result is
    /// the original BigUint.
    #[inline]
    pub fn mul_add(&mut self, multiplicator: u32, addition: u32) -> Result<(), BigUintErr> {
        let mut carry = 0u64;

        {
            let mut iter = self.chunks.iter_mut().rev();

            if let Some(chunk) = iter.next() {
                carry = u64::from(*chunk) * u64::from(multiplicator) + u64::from(addition);
                *chunk = carry as u32;
                carry >>= 32;
            }

            for chunk in iter {
                carry += u64::from(*chunk) * u64::from(multiplicator);
                *chunk = carry as u32;
                carry >>= 32;
            }
        }

        if carry > 0 {
            self.len = self.len + 1;
            if self.len > self.chunks.len() {
                return Err(BigUintErr::BackingArrayTooSmall);
            }
            for x in self.len..0 {
                self.chunks.swap(x, x - 1)
            }
            self.chunks[0] = carry as u32;
        }
        Ok(())
    }

    /// Check if self is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.chunks.iter().all(|chunk| *chunk == 0)
    }

    #[inline]
    pub fn into_bytes_be<'b>(mut self, output: &'b mut [u8]) -> Result<(), (usize, usize)> {
        let mut skip = 0;

        for chunk in self.chunks.iter() {
            if *chunk != 0 {
                skip += chunk.leading_zeros() / 8;
                break;
            }

            skip += 4;
        }

        let len = self.chunks.len() * 4 - skip as usize;
        if len == 0 {
            return Ok(());
        }
        if output.len() < len {
            return Err((output.len(), len));
        }

        for chunk in self.chunks.iter_mut() {
            *chunk = u32::to_be(*chunk);
        }

        //TODO once caculations with const generics transmute::<[u32; N], [u8; N*4]>(self.chunks) https://hackmd.io/OZG_XiLFRs2Xmw5s39jRzA?view
        unsafe {
            let chunks_ptr = (self.chunks.as_ptr() as *const u8).offset(skip as isize);
            copy_nonoverlapping(chunks_ptr, output.as_mut_ptr(), len);
        }
        Ok(())
    }

    #[inline]
    pub fn from_bytes_be(bytes: &[u8]) -> Result<Self, (usize, usize)> {
        let modulo = bytes.len() % 4;
        let mut chunks = [0u32; N];
        let len = bytes.len() / 4 + (modulo > 0) as usize;
        if bytes.len() > len * 4 {
            return Err((bytes.len(), len));
        }

        // TODO use Vec.reserve
        unsafe {
            let mut chunks_ptr = chunks.as_mut_ptr() as *mut u8;

            if modulo > 0 {
                *chunks.get_unchecked_mut(0) = 0u32;
                chunks_ptr = chunks_ptr.offset(4 - modulo as isize);
            }

            copy_nonoverlapping(bytes.as_ptr(), chunks_ptr, bytes.len());
        }

        for chunk in chunks.iter_mut() {
            *chunk = u32::from_be(*chunk);
        }

        Ok(Self { chunks, len: 0 })
    }
}

#[derive(Debug)]
pub enum BigUintErr {
    BackingArrayTooSmall,
    OutputSliceTooSmall,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::BigUintStatic;

    #[test]
    fn big_uint_static_from_bytes() {
        let bytes: &[u8] = &[
            0xDE, 0xAD, 0x00, 0x00, 0x00, 0x13, 0x37, 0xAD, 0x00, 0x00, 0x00, 0x00, 0xDE, 0xAD,
        ];
        let big = BigUintStatic::from_bytes_be(bytes).unwrap();
        let a = big;
        let mut output = [0u8; 14];
        a.into_bytes_be(&mut output).unwrap();

        assert_eq!(output, bytes);
        assert_eq!(
            big.chunks,
            [0x0000DEAD, 0x00000013, 0x37AD0000, 0x0000DEAD],
            "{:X?}:{:X?}",
            big.chunks,
            [0x0000DEAD, 0x00000013, 0x37AD0000, 0xDEAD0000u32],
        );
    }

    #[test]
    fn big_uint_static_rem_div() {
        let mut big = BigUintStatic {
            chunks: [0x136AD712, 0x84322759],
            len: 2,
        };

        let rem = big.div_mod(58);
        let merged = (u64::from(big.chunks[0]) << 32) | u64::from(big.chunks[1]);

        assert_eq!(merged, 0x136AD71284322759 / 58);
        assert_eq!(u64::from(rem), 0x136AD71284322759 % 58);
    }

    #[test]
    fn big_uint_static_add_mul() {
        let mut big = BigUintStatic {
            chunks: [0x000AD712, 0x84322759],
            len: 2,
        };

        big.mul_add(58, 37).unwrap();
        let merged = (u64::from(big.chunks[0]) << 32) | u64::from(big.chunks[1]);

        assert_eq!(merged, (0x000AD71284322759 * 58) + 37);
    }
}
