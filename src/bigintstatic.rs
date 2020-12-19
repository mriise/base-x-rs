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
pub struct BigUintStatic<const N: usize> {
    chunks: [u32; N],
    len: usize
}

impl<const N: usize> Default for BigUintStatic<N> {
    fn default() -> Self {
        Self { chunks: [0u32; N], len: 0 }
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
            if self.len > self.chunks.len() { return Err(BigUintErr::BackingArrayTooSmall) }
            for x in self.len..0 {
                self.chunks.swap(x, x-1)
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
    pub fn into_bytes_be(mut self, output: &mut [u8]) -> Result<(), BigUintErr> {
        if output.len() < self.len*4 { return Err(BigUintErr::OutputSliceTooSmall) }
        let mut skip = 0;

        for chunk in self.chunks.iter() {
            if *chunk != 0 {
                skip += chunk.leading_zeros() / 8;
                break;
            }

            skip += 4;
        }

        let len = self.chunks.len() as u32 * 4 - skip;

        if len == 0 {
            for i in 0..(self.len*4) {
                output[i] = 0;
            }
        }

        for index in skip..len {
            // SAFETY: u32 can be safely transmuted into a 4 byte array
            let bytes: [u8; 4] = self.chunks[index as usize].to_be_bytes();
            for (byte_index, byte) in bytes.iter().enumerate() {
                output[(((index-skip)*4) as usize) + byte_index] = *byte;
            }
        }
        Ok(())
    }

    #[inline]
    pub fn from_bytes_be(bytes: &[u8]) -> Result<Self, BigUintErr> {
        let remainder = bytes.len() % 4;
        let full_chunks = bytes.len() / 4;
        let last_chunk = full_chunks + (remainder > 0) as usize;

        let mut backing_array = [0u32; N];

        if last_chunk > N { return Err(BigUintErr::BackingArrayTooSmall) } // TODO: use result
        
        for i in 0..full_chunks {
            let chunk: [u8; 4] = [
                bytes[i*4],
                bytes[i*4+1],
                bytes[i*4+2],
                bytes[i*4+3]
            ];
            backing_array[i] = u32::from_be_bytes(chunk);
        }
    
        // this block will no-op if remainder is 0
        {
            let mut chunk = [0u8; 4];
            // convert partial u32
            for i in 0..remainder {
                chunk[i] = bytes[full_chunks*4 + i];
            }
            backing_array[last_chunk-1] = u32::from_be_bytes(chunk);
        }
    
        Ok(Self { chunks: backing_array, len: last_chunk })
    }
}

#[derive(Debug)]
pub enum BigUintErr {
    BackingArrayTooSmall,
    OutputSliceTooSmall
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unreadable_literal)]
    use super::BigUintStatic;

    #[test]
    fn big_uint_from_bytes() {
        let bytes: &[u8] = &[
            0x00, 0x00, 0xDE, 0xAD,
            0x00, 0x00, 0x00, 0x13,
            0x37, 0xAD, 0x00, 0x00,  
            0xDE, 0xAD, 
        ];
        let big = BigUintStatic::from_bytes_be(bytes).unwrap();

        assert_eq!(
            big.chunks,
            [0x0000DEAD, 0x00000013, 0x37AD0000, 0xDEAD0000]
        );
    }

    #[test]
    fn big_uint_rem_div() {
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
    fn big_uint_add_mul() {
        let mut big = BigUintStatic {
            chunks: [0x000AD712, 0x84322759],
            len: 2,
        };

        big.mul_add(58, 37);
        let merged = (u64::from(big.chunks[0]) << 32) | u64::from(big.chunks[1]);

        assert_eq!(merged, (0x000AD71284322759 * 58) + 37);
    }
}
