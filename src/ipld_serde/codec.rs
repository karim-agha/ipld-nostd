// //! Implementation of ipld-core's `Codec` trait.

// use {
// 	super::{de::Deserializer, error::CodecError},
// 	crate::{
// 		cid::Cid,
// 		ipld::{
// 			codec::{Codec, Links},
// 			serde::ExtractLinks,
// 		},
// 	},
// 	core2::io::{BufRead, Write},
// 	serde::{de::Deserialize, ser::Serialize},
// };

// /// DAG-CBOR implementation of ipld-core's `Codec` trait.
// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// pub struct DagCborCodec;

// impl<T> Codec<T> for DagCborCodec
// where
// 	T: for<'a> Deserialize<'a> + Serialize,
// {
// 	type Error = CodecError;

// 	const CODE: u64 = 0x71;

// 	fn decode<R: BufRead>(reader: R) -> Result<T, Self::Error> {
// 		Ok(super::from_reader(reader)?)
// 	}

// 	fn encode<W: Write>(writer: W, data: &T) -> Result<(), Self::Error> {
// 		Ok(super::to_writer(writer, data)?)
// 	}
// }

// impl Links for DagCborCodec {
// 	type LinksError = CodecError;

// 	fn links(data: &[u8]) -> Result<impl Iterator<Item = Cid>, Self::LinksError>
// { 		let mut deserializer = Deserializer::from_slice(data);
// 		Ok(
// 			ExtractLinks::deserialize(&mut deserializer)?
// 				.into_vec()
// 				.into_iter(),
// 		)
// 	}
// }
