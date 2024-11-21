use {
	super::error::Error,
	crate::{
		alloc::{borrow::ToOwned, string::ToString},
		cid::Cid,
		dag,
	},
	alloc::vec::Vec,
	serde::{Deserialize, Serialize},
};

/// A car header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CarHeader {
	V1(CarHeaderV1),
}

impl CarHeader {
	pub fn new_v1(roots: Vec<Cid>) -> Self {
		Self::V1(roots.into())
	}

	pub fn decode(buffer: &[u8]) -> Result<Self, Error> {
		let header: CarHeaderV1 =
			dag::from_slice(buffer).map_err(|e| Error::Parsing(e.to_string()))?;

		if header.roots.is_empty() {
			return Err(Error::Parsing("empty CAR file".to_owned()));
		}

		if header.version != 1 {
			return Err(Error::InvalidFile(
				"Only CAR file version 1 is supported".to_string(),
			));
		}

		Ok(CarHeader::V1(header))
	}

	pub fn encode(&self) -> Result<Vec<u8>, Error> {
		match self {
			CarHeader::V1(ref header) => {
				let res = dag::to_vec(header).expect("vec");
				Ok(res)
			}
		}
	}

	pub fn roots(&self) -> &[Cid] {
		match self {
			CarHeader::V1(header) => &header.roots,
		}
	}

	pub fn version(&self) -> u64 {
		match self {
			CarHeader::V1(_) => 1,
		}
	}
}

/// CAR file header version 1.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CarHeaderV1 {
	pub roots: Vec<Cid>,
	pub version: u64,
}

impl CarHeaderV1 {
	/// Creates a new CAR file header
	pub fn new(roots: Vec<Cid>, version: u64) -> Self {
		Self { roots, version }
	}
}

impl From<Vec<Cid>> for CarHeaderV1 {
	fn from(roots: Vec<Cid>) -> Self {
		Self { roots, version: 1 }
	}
}

#[cfg(test)]
mod tests {
	use {super::*, crate::multihash::Multihash, ::alloc::*};

	#[test]
	fn symmetric_header_v1() {
		let digest =
			Multihash::wrap(0x1e, blake3::hash(b"test").as_bytes()).unwrap();
		let cid = Cid::new_v1(0x71, digest);

		let header = CarHeaderV1::from(vec![cid]);

		let bytes = dag::to_vec(&header).unwrap();

		assert_eq!(dag::from_slice::<CarHeaderV1>(&bytes).unwrap(), header);
	}
}
