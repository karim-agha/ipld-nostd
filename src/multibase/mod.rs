//! # multibase
//!
//! Implementation of [multibase](https://github.com/multiformats/multibase) in Rust.

use alloc::{string::String, vec::Vec};

mod base;
mod encoding;
mod error;
mod impls;

pub use self::{
	base::Base,
	error::{Error, Result},
};

/// Decode the base string.
///
/// # Examples
///
/// ```
/// use multibase::{decode, Base};
///
/// assert_eq!(
/// 	decode("zCn8eVZg").unwrap(),
/// 	(Base::Base58Btc, b"hello".to_vec())
/// );
/// ```
pub fn decode<T: AsRef<str>>(input: T) -> Result<(Base, Vec<u8>)> {
	let input = input.as_ref();
	let code = input.chars().next().ok_or(Error::InvalidBaseString)?;
	let base = Base::from_code(code)?;
	let decoded = base.decode(&input[code.len_utf8()..])?;
	Ok((base, decoded))
}

/// Encode with the given byte slice to base string.
///
/// # Examples
///
/// ```
/// use multibase::{encode, Base};
///
/// assert_eq!(encode(Base::Base58Btc, b"hello"), "zCn8eVZg");
/// ```
pub fn encode<T: AsRef<[u8]>>(base: Base, input: T) -> String {
	let input = input.as_ref();
	let mut encoded = base.encode(input.as_ref());
	encoded.insert(0, base.code());
	encoded
}
