//! When serializing or deserializing DAG-CBOR goes wrong.

use {
	alloc::{
		collections::TryReserveError,
		string::{String, ToString},
	},
	core::{convert::Infallible, fmt, num::TryFromIntError},
	serde::{de, ser},
};

/// An encoding error.
#[derive(Debug)]
pub enum EncodeError<E> {
	/// Custom error message.
	Msg(String),
	/// IO Error.
	Write(E),
}

impl<E> From<E> for EncodeError<E> {
	fn from(err: E) -> EncodeError<E> {
		EncodeError::Write(err)
	}
}

impl<E: fmt::Debug> ser::Error for EncodeError<E> {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		EncodeError::Msg(msg.to_string())
	}
}

impl<E: fmt::Debug> ser::StdError for EncodeError<E> {}

impl<E: fmt::Debug> fmt::Display for EncodeError<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl<E: fmt::Debug> From<cbor4ii::EncodeError<E>> for EncodeError<E> {
	fn from(err: cbor4ii::EncodeError<E>) -> EncodeError<E> {
		match err {
			cbor4ii::EncodeError::Write(e) => EncodeError::Write(e),
			// Needed as `cbor4ii::EncodeError` is markes as non_exhaustive
			_ => EncodeError::Msg(err.to_string()),
		}
	}
}

/// A decoding error.
#[derive(Debug)]
pub enum DecodeError<E> {
	/// Custom error message.
	Msg(String),
	/// IO error.
	Read(E),
	/// End of file.
	Eof,
	/// Unexpected byte.
	Mismatch {
		/// Expected CBOR major type.
		expect_major: u8,
		/// Unexpected byte.
		byte: u8,
	},
	/// Unexpected type.
	TypeMismatch {
		/// Type name.
		name: &'static str,
		/// Type byte.
		byte: u8,
	},
	/// Too large integer.
	CastOverflow(TryFromIntError),
	/// Overflowing 128-bit integers.
	Overflow {
		/// Type of integer.
		name: &'static str,
	},
	/// Decoding bytes/strings might require a borrow.
	RequireBorrowed {
		/// Type name (e.g. "bytes", "str").
		name: &'static str,
	},
	/// Length wasn't large enough.
	RequireLength {
		/// Type name.
		name: &'static str,
		/// Required length.
		expect: usize,
		/// Given length.
		value: usize,
	},
	/// Invalid UTF-8.
	InvalidUtf8(core::str::Utf8Error),
	/// Unsupported byte.
	Unsupported {
		/// Unsupported bute.
		byte: u8,
	},
	/// Recursion limit reached.
	DepthLimit,
	/// Trailing data.
	TrailingData,
	/// Indefinite sized item was encountered.
	IndefiniteSize,
}

impl<E> From<E> for DecodeError<E> {
	fn from(err: E) -> DecodeError<E> {
		DecodeError::Read(err)
	}
}

impl<E: fmt::Debug> de::Error for DecodeError<E> {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		DecodeError::Msg(msg.to_string())
	}
}

impl<E: fmt::Debug> ser::StdError for DecodeError<E> {}

impl<E: fmt::Debug> fmt::Display for DecodeError<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl<E: fmt::Debug> From<cbor4ii::DecodeError<E>> for DecodeError<E> {
	fn from(err: cbor4ii::DecodeError<E>) -> DecodeError<E> {
		match err {
			cbor4ii::DecodeError::Read(read) => DecodeError::Read(read),
			cbor4ii::DecodeError::Eof => DecodeError::Eof,
			cbor4ii::DecodeError::Mismatch { expect_major, byte } => {
				DecodeError::Mismatch { expect_major, byte }
			}
			cbor4ii::DecodeError::TypeMismatch { name, byte } => {
				DecodeError::TypeMismatch { name, byte }
			}
			cbor4ii::DecodeError::CastOverflow(overflow) => {
				DecodeError::CastOverflow(overflow)
			}
			cbor4ii::DecodeError::Overflow { name } => DecodeError::Overflow { name },
			cbor4ii::DecodeError::RequireBorrowed { name } => {
				DecodeError::RequireBorrowed { name }
			}
			cbor4ii::DecodeError::RequireLength {
				name,
				expect,
				value,
			} => DecodeError::RequireLength {
				name,
				expect,
				value,
			},
			cbor4ii::DecodeError::InvalidUtf8(invalid) => {
				DecodeError::InvalidUtf8(invalid)
			}
			cbor4ii::DecodeError::Unsupported { byte } => {
				DecodeError::Unsupported { byte }
			}
			cbor4ii::DecodeError::DepthLimit => DecodeError::DepthLimit,
			// Needed as `cbor4ii::EncodeError` is markes as non_exhaustive
			_ => DecodeError::Msg(err.to_string()),
		}
	}
}

/// Encode and Decode error combined.
#[derive(Debug, thiserror_core2::Error)]
pub enum CodecError {
	/// A decoding error.
	#[error("Decoding error: {0}")]
	Decode(DecodeError<Infallible>),
	/// An encoding error.
	#[error("Encoding error: {0}")]
	Encode(EncodeError<TryReserveError>),
	/// A decoding error.
	#[error("Decoding IO error: {0}")]
	DecodeIo(DecodeError<core2::io::Error>),
	/// An encoding error.
	#[error("Encoding IO error: {0}")]
	EncodeIo(EncodeError<core2::io::Error>),
}

impl From<DecodeError<Infallible>> for CodecError {
	fn from(error: DecodeError<Infallible>) -> Self {
		Self::Decode(error)
	}
}

impl From<EncodeError<TryReserveError>> for CodecError {
	fn from(error: EncodeError<TryReserveError>) -> Self {
		Self::Encode(error)
	}
}
