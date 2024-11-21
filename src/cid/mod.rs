//! # cid
//!
//! Implementation of [cid](https://github.com/ipld/cid) in Rust.

mod cid;
mod error;
pub mod serde;
mod version;

pub use self::{
	cid::Cid as CidGeneric,
	error::{Error, Result},
	version::Version,
};

/// A Cid that contains a multihash with an allocated size of 512 bits.
///
/// This is the same digest size the default multihash code table has.
///
/// If you need a CID that is generic over its digest size, use [`CidGeneric`]
/// instead.
pub type Cid = CidGeneric<64>;
