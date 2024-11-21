// Copyright 2020 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(dead_code)]

use {core::fmt, core2::io};

macro_rules! gen {
    ($($name:ident, $d:expr, $t:ident, $b:ident);*) => {
        $(
            pub fn $name<R: io::Read>(mut reader: R) -> Result<$t, ReadError> {
                let mut b = encode::$b();
                for i in 0 .. b.len() {
                    let n = reader.read(&mut b[i .. i + 1])?;
                    if n == 0 {
                        return Err(ReadError::Io(io::ErrorKind::UnexpectedEof.into()))
                    }
                    if decode::is_last(b[i]) {
                        return Ok(decode::$t(&b[..= i])?.0)
                    }
                }
                Err(decode::Error::Overflow.into())
            }
        )*
    }
}

gen! {
		read_u8,    "`u8`",    u8,    u8_buffer;
		read_u16,   "`u16`",   u16,   u16_buffer;
		read_u32,   "`u32`",   u32,   u32_buffer;
		read_u64,   "`u64`",   u64,   u64_buffer;
		read_u128,  "`u128`",  u128,  u128_buffer;
		read_usize, "`usize`", usize, usize_buffer
}

/// Possible read errors.
#[non_exhaustive]
#[derive(Debug)]
pub enum ReadError {
	Io(io::Error),
	Decode(decode::Error),
}

impl fmt::Display for ReadError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ReadError::Io(e) => write!(f, "i/o error: {}", e),
			ReadError::Decode(e) => write!(f, "decode error: {}", e),
		}
	}
}

impl core2::error::Error for ReadError {
	fn source(&self) -> Option<&(dyn core2::error::Error + 'static)> {
		if let ReadError::Io(e) = self {
			Some(e)
		} else {
			None
		}
	}
}

impl From<io::Error> for ReadError {
	fn from(e: io::Error) -> Self {
		ReadError::Io(e)
	}
}

impl From<decode::Error> for ReadError {
	fn from(e: decode::Error) -> Self {
		ReadError::Decode(e)
	}
}

impl From<ReadError> for io::Error {
	fn from(val: ReadError) -> Self {
		match val {
			ReadError::Io(e) => e,
			ReadError::Decode(e) => e.into(),
		}
	}
}

pub mod decode {
	use core::fmt;

	/// Possible decoding errors.
	///
	/// **Note**: The `std` feature is required for the `std::error::Error` impl
	/// and the conversion to `std::io::Error`.
	#[non_exhaustive]
	#[derive(Clone, Debug, PartialEq, Eq)]
	pub enum Error {
		/// Not enough input bytes.
		Insufficient,
		/// Input bytes exceed maximum.
		Overflow,
		/// Encoding is not minimal (has trailing zero bytes).
		NotMinimal,
	}

	impl fmt::Display for Error {
		fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
			match self {
				Error::Insufficient => f.write_str("not enough input bytes"),
				Error::Overflow => f.write_str("input bytes exceed maximum"),
				Error::NotMinimal => f.write_str("encoding is not minimal"),
			}
		}
	}

	/// Only available when the feature `std` is present.
	impl core2::error::Error for Error {}

	/// Only available when the feature `std` is present.
	impl From<Error> for core2::io::Error {
		fn from(val: Error) -> Self {
			let kind = match val {
				Error::Insufficient => core2::io::ErrorKind::UnexpectedEof,
				Error::Overflow => core2::io::ErrorKind::InvalidData,
				Error::NotMinimal => core2::io::ErrorKind::InvalidData,
			};
			core2::io::Error::new(kind, "varint decoding error")
		}
	}

	macro_rules! decode {
		($buf:expr, $max_bytes:expr, $typ:ident) => {{
			let mut n = 0;
			for (i, b) in $buf.iter().cloned().enumerate() {
				let k = $typ::from(b & 0x7F);
				n |= k << (i * 7);
				if is_last(b) {
					if b == 0 && i > 0 {
						// If last byte (of a multi-byte varint) is zero, it could have been
						// "more minimally" encoded by dropping that trailing zero.
						return Err(Error::NotMinimal);
					}
					return Ok((n, &$buf[i + 1..]));
				}
				if i == $max_bytes {
					return Err(Error::Overflow);
				}
			}
			Err(Error::Insufficient)
		}};
	}

	/// Is this the last byte of an unsigned varint?
	#[inline]
	pub fn is_last(b: u8) -> bool {
		b & 0x80 == 0
	}

	/// Decode the given slice as `u8`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	pub fn u8(buf: &[u8]) -> Result<(u8, &[u8]), Error> {
		decode!(buf, 1, u8)
	}

	/// Decode the given slice as `u16`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	pub fn u16(buf: &[u8]) -> Result<(u16, &[u8]), Error> {
		decode!(buf, 2, u16)
	}

	/// Decode the given slice as `u32`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	pub fn u32(buf: &[u8]) -> Result<(u32, &[u8]), Error> {
		decode!(buf, 4, u32)
	}

	/// Decode the given slice as `u64`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	pub fn u64(buf: &[u8]) -> Result<(u64, &[u8]), Error> {
		decode!(buf, 9, u64)
	}

	/// Decode the given slice as `u128`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	pub fn u128(buf: &[u8]) -> Result<(u128, &[u8]), Error> {
		decode!(buf, 18, u128)
	}

	/// Decode the given slice as `usize`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	#[cfg(target_pointer_width = "64")]
	pub fn usize(buf: &[u8]) -> Result<(usize, &[u8]), Error> {
		u64(buf).map(|(n, i)| (n as usize, i))
	}

	/// Decode the given slice as `usize`.
	///
	/// Returns the value and the remaining slice.
	#[inline]
	#[cfg(target_pointer_width = "32")]
	pub fn usize(buf: &[u8]) -> Result<(usize, &[u8]), Error> {
		u32(buf).map(|(n, i)| (n as usize, i))
	}
}

pub mod encode {

	macro_rules! encode {
		($number:expr, $buf:expr) => {{
			let mut n = $number;
			let mut i = 0;
			for b in $buf.iter_mut() {
				*b = n as u8 | 0x80;
				n >>= 7;
				if n == 0 {
					*b &= 0x7f;
					break;
				}
				i += 1
			}
			debug_assert_eq!(n, 0);
			&$buf[0..=i]
		}};
	}

	/// Encode the given `u8` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	pub fn u8(number: u8, buf: &mut [u8; U8_LEN]) -> &[u8] {
		encode!(number, buf)
	}

	/// Encode the given `u16` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	pub fn u16(number: u16, buf: &mut [u8; U16_LEN]) -> &[u8] {
		encode!(number, buf)
	}

	/// Encode the given `u32` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	pub fn u32(number: u32, buf: &mut [u8; U32_LEN]) -> &[u8] {
		encode!(number, buf)
	}

	/// Encode the given `u64` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	pub fn u64(number: u64, buf: &mut [u8; U64_LEN]) -> &[u8] {
		encode!(number, buf)
	}

	/// Encode the given `u128` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	pub fn u128(number: u128, buf: &mut [u8; U128_LEN]) -> &[u8] {
		encode!(number, buf)
	}

	/// Encode the given `usize` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	#[cfg(target_pointer_width = "64")]
	pub fn usize(number: usize, buf: &mut [u8; USIZE_LEN]) -> &[u8] {
		u64(number as u64, buf)
	}

	/// Encode the given `usize` into the given byte array.
	///
	/// Returns the slice of encoded bytes.
	#[inline]
	#[cfg(target_pointer_width = "32")]
	pub fn usize(number: usize, buf: &mut [u8; USIZE_LEN]) -> &[u8] {
		u32(number as u32, buf)
	}

	/// Create new array buffer for encoding of `u8` values.
	#[inline]
	pub fn u8_buffer() -> [u8; U8_LEN] {
		[0; U8_LEN]
	}

	/// Create new array buffer for encoding of `u16` values.
	#[inline]
	pub fn u16_buffer() -> [u8; U16_LEN] {
		[0; U16_LEN]
	}

	/// Create new array buffer for encoding of `u32` values.
	#[inline]
	pub fn u32_buffer() -> [u8; U32_LEN] {
		[0; U32_LEN]
	}

	/// Create new array buffer for encoding of `u64` values.
	#[inline]
	pub fn u64_buffer() -> [u8; U64_LEN] {
		[0; U64_LEN]
	}

	/// Create new array buffer for encoding of `u128` values.
	#[inline]
	pub fn u128_buffer() -> [u8; U128_LEN] {
		[0; U128_LEN]
	}

	/// Create new array buffer for encoding of `usize` values.
	#[inline]
	pub fn usize_buffer() -> [u8; USIZE_LEN] {
		[0; USIZE_LEN]
	}

	// Required lengths of encoding buffers:

	const U8_LEN: usize = 2;
	const U16_LEN: usize = 3;
	const U32_LEN: usize = 5;
	const U64_LEN: usize = 10;
	const U128_LEN: usize = 19;

	#[cfg(target_pointer_width = "64")]
	const USIZE_LEN: usize = U64_LEN;

	#[cfg(target_pointer_width = "32")]
	const USIZE_LEN: usize = U32_LEN;
}
