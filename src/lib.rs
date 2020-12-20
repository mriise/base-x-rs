//! # base_x
//!
//! Encode and decode any base alphabet.
//!
//! ## Installation
//!
//! Add this to `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! base-x = "0.2.0"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! extern crate base_x;
//!
//! fn main() {
//!   let decoded = base_x::decode("01", "11111111000000001111111100000000").unwrap();
//!   let encoded = base_x::encode("01", &decoded);
//!  assert_eq!(encoded, "11111111000000001111111100000000");
//! }
//! ```
#![feature(const_fn_floating_point_arithmetic)]
#![feature(min_const_generics)]
#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// #[cfg(feature = "alloc")]
// use alloc::{string::String, vec::Vec};

pub mod alphabet;
#[cfg(feature = "alloc")]
mod bigint;
mod bigintstatic;
pub mod decoder;
pub mod encoder;

pub use alphabet::Alphabet;

#[derive(Debug)]
pub struct DecodeError;

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Failed to decode the given data")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {
    fn description(&self) -> &str {
        "Can not decode the provided data"
    }
}

#[cfg(feature = "alloc")]
/// Encode an input vector using the given alphabet.
pub fn encode<A: Alphabet>(alphabet: A, input: &[u8]) -> alloc::string::String {
    alphabet.encode(input)
}

#[cfg(feature = "alloc")]
/// Decode an input vector using the given alphabet.
pub fn decode<A: Alphabet>(alphabet: A, input: &str) -> Result<alloc::vec::Vec<u8>, DecodeError> {
    alphabet.decode(input)
}

/// Encode an input vector using the given alphabet.
pub fn encode_mut<A: Alphabet, const BACKING: usize>(alphabet: A, output: &mut [u8], input: &[u8]) -> Result<(), ()> {
    alphabet.encode_mut::<BACKING>(output, input)
}

pub fn decode_mut<A: Alphabet, const BACKING: usize>(alphabet: A, output: &mut [u8], input: &str) -> Result<(), DecodeError> {
    alphabet.decode_mut::<BACKING>(output, input)
}

pub const fn gen_backing_size(base: usize, input_byte_size: usize) -> usize {
    let max_bytes = gen_encoded_size(base, input_byte_size);
    max_bytes / 4 + ((max_bytes % 4 > 0) as usize)
}

pub const fn gen_encoded_size(base: usize, input_byte_size: usize) -> usize {
    (input_byte_size as f64 * (log10(256) / log10(base))) as usize + 1
}

pub const fn gen_decoded_size(base: usize, input_byte_size: usize) -> usize {
    (input_byte_size as f64 * (log10(base) / log10(256))) as usize //might need to + 1 here maybe
}

// https://stackoverflow.com/questions/35968963/trying-to-calculate-logarithm-base-10-without-math-h-really-close-just-having
const fn ln(x: usize) -> f64 {
    let mut old_sum = 0.0;
    let xmlxpl = (x as f64 - 1.0) / (x as f64 + 1.0);
    let xmlxpl_2 = xmlxpl * xmlxpl;
    let mut denom = 1.0;
    let mut frac = xmlxpl;
    let term = frac;
    let mut sum = term;

    while sum != old_sum {
        old_sum = sum;
        denom += 2.0;
        frac *= xmlxpl_2;
        sum += frac / denom;
    }
    return 2.0 * sum;
}
const LN10: f64 = 2.3025850929940456840179914546844;
const fn log10(x: usize) -> f64 {
    return ln(x) / LN10;
}

#[cfg(feature = "std")]
#[cfg(test)]
mod test {

    use super::decode;
    use super::encode;
    use super::{gen_decoded_size, gen_encoded_size};

    extern crate json;
    use self::json::parse;
    
    use std::fs::File;
    use std::io::Read;
    use std::string::String;

    #[test]
    fn works() {
        let mut file = File::open("./fixtures/fixtures.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json = parse(&data).unwrap();
        let alphabets = &json["alphabets"];

        for value in json["valid"].members() {
            let alphabet_name = value["alphabet"].as_str().unwrap();
            let input = value["string"].as_str().unwrap();
            let alphabet = alphabets[alphabet_name].as_str().unwrap();

            // Alphabet works as unicode
            let decoded = decode(alphabet, input).unwrap();
            let encoded = encode(alphabet, &decoded);
            assert_eq!(encoded, input);

            // Alphabet works as ASCII
            let decoded = decode(alphabet.as_bytes(), input).unwrap();
            let encoded = encode(alphabet.as_bytes(), &decoded);
            assert_eq!(encoded, input);
        }
    }

    #[test]
    fn gen_works() {
        let input = [
            0xac, 0x77, 0x81, 0x4b, 0xa4, 0xcd, 0xb7, 0xb8, 0x29, 0x9d, 0x6e, 0x38, 0x94, 0x40,
            0x53, 0xbf, 0x01, 0x96, 0x2b, 0xb3, 0xdd, 0x7b, 0x39, 0x81, 0x98, 0xcc, 0x4d, 0x43,
            0x9d, 0x95, 0x1a, 0xdd, 0xb8, 0x49, 0x21, 0xeb, 0xf3, 0x2a, 0x60, 0xbc, 0xd8, 0x4f,
            0xc4, 0xe6, 0x01, 0x59, 0x90, 0x1b, 0x41, 0xec, 0x67, 0x90, 0x30, 0x96, 0xfe, 0x20,
            0x43, 0xa9, 0xf3, 0xb7, 0x97, 0xfe, 0xce, 0x7e, 0x40, 0x67, 0xec, 0xeb, 0x17, 0xa8,
            0x0d, 0xd4, 0xf7, 0xe9, 0x3d, 0xa8, 0x9f, 0x87, 0x22, 0xbc, 0x69, 0xd4, 0x19, 0x50,
            0xb2, 0x99, 0x94, 0x4b, 0xd1, 0x45, 0x68, 0x96, 0xbf, 0x6a, 0x8d, 0x42, 0x3b, 0x6c,
            0x03, 0xc5, 0xa3, 0x78, 0x80, 0x1f, 0x50, 0x8b, 0xca, 0x99, 0x9d, 0x82, 0x19, 0x82,
            0x05, 0x47, 0x9c, 0x21, 0x5d, 0x24, 0xb3, 0x94, 0x9d, 0x1a, 0x89, 0xe6, 0x27, 0x48,
            0x00, 0x15, 0xbb, 0xcc, 0x6f, 0x37, 0x66, 0x13, 0x3f, 0x21, 0x10, 0xf2, 0x58, 0x51,
            0xb0, 0x9d, 0x55, 0x83, 0x41, 0xda, 0xb8, 0xb4, 0xd8, 0x60, 0xc2, 0x64, 0xc6, 0xb8,
            0x56, 0x7f, 0x5d, 0x1d, 0xae, 0xc1, 0x05, 0x39, 0x3e, 0x59, 0x2c, 0x93, 0x9c, 0x10,
            0x42, 0x86, 0xcf, 0xe5, 0x5d, 0x36, 0xd6, 0x61, 0xbb, 0x4f, 0xea, 0x0c, 0x53, 0xd5,
            0xcd, 0xab, 0x76, 0x18, 0x38, 0xb5, 0xf8, 0x10, 0x20, 0x86, 0x55, 0x89, 0x3e, 0x7e,
            0xb3, 0x29, 0x84, 0x16, 0x6d, 0xde, 0xb6, 0xf4, 0xfd, 0xc9, 0x26, 0xe3, 0xa3, 0x59,
            0x69, 0x84, 0x07, 0xad, 0x16, 0xc9, 0x32, 0xaf, 0x1c, 0xba, 0x28, 0xb9, 0xd3, 0xd2,
            0x1f, 0xbf, 0x8c, 0xee, 0x7b, 0x8f, 0xe4, 0xe9, 0x21, 0xb0, 0x9c, 0x47, 0x62, 0xfa,
            0x38, 0x6f, 0xfc, 0xaf, 0xc2, 0xec, 0xbd, 0xe0, 0x3c, 0x7f, 0x7d, 0xba, 0x03, 0xec,
            0x2c, 0x4d, 0x03, 0x21,
        ];

        let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        let encoded = encode(alphabet, &input);
        let decoded = decode(alphabet, &encoded).unwrap();
        assert_eq!(encoded.len(), gen_encoded_size(58, input.len()));
        assert_eq!(
            decoded.len(),
            gen_decoded_size(58, gen_encoded_size(58, input.len()))
        );
    }
    
    #[test]
    fn no_std_works() {
        //todo: convert test cases into pure rust
        struct TestCases<'a> {
            alphabet: &'a str,
            // src, output
            valids: &'a [(&'a [u8],  &'a str)],
            invalids: &'a [&'a [u8]]
        }
    }

    #[test]
    fn is_unicode_sound() {
        // binary, kinda...
        let alphabet = "😐😀";

        let encoded = encode(alphabet, &[0xff, 0x00, 0xff, 0x00]);
        let decoded = decode(alphabet, &encoded).unwrap();

        assert_eq!(
            encoded,
            "😀😀😀😀😀😀😀😀😐😐😐😐😐😐😐😐😀😀😀😀😀😀😀😀😐😐😐😐😐😐😐😐"
        );
        assert_eq!(decoded, &[0xff, 0x00, 0xff, 0x00]);
    }
}
