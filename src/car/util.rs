use super::error::Error;
use crate::{cid::Cid, varint};
use alloc::{string::ToString, vec::Vec};
use core2::io::{Read, Write};

/// Maximum size that is used for single node.
pub(crate) const MAX_ALLOC: usize = 4 * 1024 * 1024;

pub(crate) fn ld_read<R>(mut reader: R, buf: &mut Vec<u8>) -> Result<Option<&[u8]>, Error>
where
    R: Read,
{
    let length: usize = match read_varint_usize(&mut reader) {
        Ok(Some(len)) => len,
        Ok(None) => return Ok(None),
        Err(e) => {
            return Err(Error::Parsing(e.to_string()));
        }
    };
    if length > MAX_ALLOC {
        return Err(Error::LdReadTooLarge(length));
    }
    if length > buf.len() {
        buf.resize(length, 0);
    }

    reader
        .read_exact(&mut buf[..length])
        .map_err(|e| Error::Parsing(e.to_string()))?;

    Ok(Some(&buf[..length]))
}

/// Read a varint from the provided reader. Returns `Ok(None)` on unexpected `EOF`.
pub fn read_varint_usize<R: Read + Unpin>(
    mut reader: R,
) -> Result<Option<usize>, varint::ReadError> {
    let mut b = varint::encode::usize_buffer();
    for i in 0..b.len() {
        let n = reader.read(&mut b[i..i + 1])?;
        if n == 0 {
            return Ok(None);
        }
        if varint::decode::is_last(b[i]) {
            let slice = &b[..=i];
            let (num, _) = varint::decode::usize(slice)?;
            return Ok(Some(num));
        }
    }
    Err(varint::decode::Error::Overflow)?
}

/// Write the given number as varint to the provided writer.
pub fn write_varint_usize<W: Write + Unpin>(num: usize, mut writer: W) -> core2::io::Result<usize> {
    let mut buffer = varint::encode::usize_buffer();
    let to_write = varint::encode::usize(num, &mut buffer);
    writer.write_all(to_write)?;

    Ok(to_write.len())
}

pub(crate) fn read_node<R>(
    buf_reader: &mut R,
    buf: &mut Vec<u8>,
) -> Result<Option<(Cid, Vec<u8>)>, Error>
where
    R: Read,
{
    if let Some(buf) = ld_read(buf_reader, buf)? {
        let mut cursor = core2::io::Cursor::new(buf);
        let c = Cid::read_bytes(&mut cursor)?;
        let pos = cursor.position() as usize;

        return Ok(Some((c, buf[pos..].to_vec())));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloc::vec;
    use core2::io::Write;

    fn ld_write<'a, W>(writer: &mut W, bytes: &[u8]) -> Result<(), Error>
    where
        W: Write,
    {
        write_varint_usize(bytes.len(), &mut *writer)?;
        writer.write_all(bytes)?;
        writer.flush()?;
        Ok(())
    }

    #[test]
    fn ld_read_write_good() {
        let mut buffer = Vec::<u8>::new();
        ld_write(&mut buffer, b"test bytes").unwrap();
        let reader = core2::io::Cursor::new(buffer);

        let mut buffer = vec![1u8; 1024];
        let read = ld_read(reader, &mut buffer).unwrap().unwrap();
        assert_eq!(read, b"test bytes");
    }

    #[test]
    fn ld_read_write_fail() {
        let mut buffer = Vec::<u8>::new();
        let size = MAX_ALLOC + 1;
        ld_write(&mut buffer, &vec![2u8; size]).unwrap();
        let reader = core2::io::Cursor::new(buffer);

        let mut buffer = vec![1u8; 1024];
        let read = ld_read(reader, &mut buffer);
        assert!(matches!(read, Err(Error::LdReadTooLarge(_))));
    }
}
