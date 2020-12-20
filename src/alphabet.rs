#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};
use DecodeError;

use decoder::*;
use encoder;

pub trait Alphabet {
    #[cfg(feature = "alloc")]
    fn encode(self, input: &[u8]) -> String;

    #[cfg(feature = "alloc")]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError>;

    fn encode_mut<const BACKING: usize>(self, output: &mut [u8], input: &[u8]) -> Result<(), ()>;

    fn decode_mut<const BACKING: usize>(self, output: &mut [u8], input: &str) -> Result<(), DecodeError>;
}

impl<'a> Alphabet for &[u8] {
    #[cfg(feature = "alloc")]
    #[inline(always)]
    fn encode(self, input: &[u8]) -> String {
        if !self.is_ascii() {
            panic!("Alphabet must be ASCII");
        }

        let mut out = encoder::encode(self, input);
        out.reverse();
        unsafe { String::from_utf8_unchecked(out) }
    }

    #[cfg(feature = "alloc")]
    #[inline(always)]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError> {
        U8Decoder::new(self).decode(input)
    }

    #[inline(always)]
    fn encode_mut<const BACKING: usize>(self, output: &mut [u8], input: &[u8]) -> Result<(), ()> {
        if !self.is_ascii() {
            panic!("Alphabet must be ASCII");
        }
        match encoder::encode_mut::<u8, BACKING>(self, output, input) {
            Ok(_) => (),
            Err(_) => return Err(())
        }
        output.reverse();
        Ok(())
    }

    #[inline(always)]
    fn decode_mut<const BACKING: usize>(self, output: &mut [u8], input: &str) -> Result<(), DecodeError> {
        match U8Decoder::new(self).decode_mut::<BACKING>(output, input) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(DecodeError)
        }
    }
}

impl<'a> Alphabet for &str {
    #[cfg(feature = "alloc")]
    #[inline(always)]
    fn encode(self, input: &[u8]) -> String {
        if self.is_ascii() {
            let mut out = encoder::encode(self.as_bytes(), input);
            out.reverse();
            unsafe { String::from_utf8_unchecked(out) }
        } else {
            let alphabet: Vec<char> = self.chars().collect();
            let out = encoder::encode(&alphabet, input);
            out.iter().rev().collect()
        }
    }

    #[cfg(feature = "alloc")]
    #[inline(always)]
    fn decode(self, input: &str) -> Result<Vec<u8>, DecodeError> {
        if self.is_ascii() {
            U8Decoder::new(self.as_bytes()).decode(input)
        } else {
            let alphabet: Vec<char> = self.chars().collect();
            CharDecoder(&alphabet).decode(input)
        }
    }

    #[inline(always)]
    fn encode_mut<const BACKING: usize>(self, output: &mut [u8], input: &[u8]) -> Result<(), ()> {
        if !self.is_ascii() {
            panic!("Alphabet must be ASCII");
        }
        match encoder::encode_mut::<u8, BACKING>(self.as_bytes(), output, input) {
            Ok(_) => (),
            Err(_) => return Err(())
        }
        output.reverse();
        Ok(())
    }

    #[inline(always)]
    fn decode_mut<const BACKING: usize>(self, output: &mut [u8], input: &str) -> Result<(), DecodeError> {
        match U8Decoder::new(self.as_bytes()).decode_mut::<BACKING>(output, input) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(DecodeError)
        }
    }
}
