use {
	super::{
		error::Error,
		header::CarHeader,
		util::{ld_read, read_node},
	},
	crate::cid::Cid,
	alloc::{string::ToString, vec::Vec},
};

/// Reads CAR files that are in a BufReader
#[derive(Debug)]
pub struct CarReader<R> {
	reader: R,
	header: CarHeader,
	buffer: Vec<u8>,
}

impl<R> CarReader<R>
where
	R: core2::io::Read,
{
	/// Creates a new CarReader and parses the CarHeader
	pub fn new(mut reader: R) -> Result<Self, Error> {
		let mut buffer = Vec::new();

		match ld_read(&mut reader, &mut buffer)? {
			Some(buf) => {
				let header = CarHeader::decode(buf)?;

				Ok(CarReader {
					reader,
					header,
					buffer,
				})
			}
			None => Err(Error::Parsing(
				"failed to parse uvarint for header".to_string(),
			)),
		}
	}

	/// Returns the header of this car file.
	pub fn header(&self) -> &CarHeader {
		&self.header
	}

	/// Returns the next IPLD Block in the buffer
	pub fn next_block(&mut self) -> Result<Option<(Cid, Vec<u8>)>, Error> {
		read_node(&mut self.reader, &mut self.buffer)
	}
}

impl<R: core2::io::Read> IntoIterator for CarReader<R> {
	type IntoIter = Iter<R>;
	type Item = Result<(Cid, Vec<u8>), Error>;

	fn into_iter(self) -> Self::IntoIter {
		Iter(self)
	}
}

pub struct Iter<R: core2::io::Read>(CarReader<R>);
impl<R: core2::io::Read> Iterator for Iter<R> {
	type Item = Result<(Cid, Vec<u8>), Error>;

	fn next(&mut self) -> Option<Self::Item> {
		read_node(&mut self.0.reader, &mut self.0.buffer).transpose()
	}
}

#[cfg(test)]
mod tests {
	use {
		super::super::{header::CarHeaderV1, writer::CarWriter, *},
		crate::{cid::Cid, multihash::Multihash},
		::alloc::{vec::Vec, *},
		core2::io::Cursor,
		itertools::Itertools,
	};

	fn car_write_read() {
		let digest_test =
			Multihash::wrap(0x1e, blake3::hash(b"test").as_bytes()).unwrap();
		let cid_test = Cid::new_v1(0x71, digest_test);
		let digest_foo =
			Multihash::wrap(0x1e, blake3::hash(b"foo").as_bytes()).unwrap();
		let cid_foo = Cid::new_v1(0x71, digest_foo);

		let header = CarHeader::V1(CarHeaderV1::from(vec![cid_foo]));

		let mut buffer = Vec::new();
		let mut writer = CarWriter::new(header, &mut buffer);
		writer.write(cid_test, b"test").unwrap();
		writer.write(cid_foo, b"foo").unwrap();
		writer.finish().unwrap();

		let reader = Cursor::new(&buffer);
		let car_reader = CarReader::new(reader).unwrap();
		let files: Vec<_> = car_reader.into_iter().try_collect().unwrap();

		assert_eq!(files.len(), 2);
		assert_eq!(files[0].0, cid_test);
		assert_eq!(files[0].1, b"test");
		assert_eq!(files[1].0, cid_foo);
		assert_eq!(files[1].1, b"foo");
	}
}
