use crate::error::Error;
use std::io::{Read, Write};

pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8, Error> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(u8::from_le_bytes(bytes))
}

pub fn write_u8<W: Write>(v: u8, writer: &mut W) -> Result<(), Error> {
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16, Error> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

pub fn write_u16<W: Write>(v: u16, writer: &mut W) -> Result<(), Error> {
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_i32<R: Read>(reader: &mut R) -> Result<i32, Error> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

pub fn write_i32<W: Write>(v: i32, writer: &mut W) -> Result<(), Error> {
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_u32<R: Read>(reader: &mut R) -> Result<u32, Error> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

pub fn write_u32<W: Write>(v: u32, writer: &mut W) -> Result<(), Error> {
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_f32<R: Read>(reader: &mut R) -> Result<f32, Error> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_le_bytes(bytes))
}

pub fn write_f32<W: Write>(v: f32, writer: &mut W) -> Result<(), Error> {
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

pub fn read_usize<R: Read>(reader: &mut R) -> Result<usize, Error> {
    read_i32(reader)?.try_into().map_err(|_| Error::InvalidSave)
}

pub fn write_usize<W: Write>(v: usize, writer: &mut W) -> Result<(), Error> {
    let v: i32 = v.try_into().map_err(|_| Error::InvalidSave)?;
    write_i32(v, writer)
}

pub fn read_string<R: Read>(reader: &mut R) -> Result<String, Error> {
    let string_len = read_usize(reader)?;
    let bytes = read_bytevec(reader, string_len)?;
    String::from_utf8(bytes).map_err(|_| Error::InvalidSave)
}

pub fn write_str<W: Write>(s: &str, writer: &mut W) -> Result<(), Error> {
    write_usize(s.len(), writer)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

pub fn read_vec<F, T>(len: usize, mut reader: F) -> Result<Vec<T>, Error>
where
    F: FnMut() -> Result<T, Error>,
{
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(reader()?);
    }
    Ok(vec)
}

pub fn read_array<F, T, const LEN: usize>(mut reader: F) -> Result<[T; LEN], Error>
where
    F: FnMut() -> Result<T, Error>,
    T: Default + Copy,
{
    let mut arr = [Default::default(); LEN];
    for slot in &mut arr {
        *slot = reader()?;
    }
    Ok(arr)
}

pub fn read_magic<R: Read>(reader: &mut R, magic_bytes: &[u8]) -> Result<(), Error> {
    let mut header = [0u8; 16];
    reader.read_exact(&mut header)?;
    if header != *magic_bytes {
        return Err(Error::InvalidSave);
    }
    Ok(())
}

pub fn read_version<R: Read>(reader: &mut R) -> Result<[i32; 4], Error> {
    read_array(|| read_i32(reader))
}

pub fn read_bytevec<R: Read>(reader: &mut R, len: usize) -> Result<Vec<u8>, Error> {
    let mut vec = vec![0u8; len];
    reader.read_exact(&mut vec)?;
    Ok(vec)
}
