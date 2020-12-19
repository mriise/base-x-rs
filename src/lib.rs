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
#![feature(min_const_generics)]

#![no_std]
#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "alloc")]
extern crate alloc;

// #[cfg(feature = "alloc")]
// use alloc::{string::String, vec::Vec};

pub mod alphabet;
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

pub const fn output_size(alphabet: &str, input_size: usize) -> usize{
    input_size
}


#[cfg(feature = "std")]
#[cfg(test)]
mod test {
    use super::decode;
    use super::encode;
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
