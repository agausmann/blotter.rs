use crate::error::Error;
use std::{
    io::{Read, Write},
    iter::repeat_with,
};

pub trait ReadFrom: Sized {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error>;
}

pub trait ReadFromSeed<Seed>: Sized {
    fn read_from_seed<R: Read>(reader: &mut R, seed: Seed) -> Result<Self, Error>;
}

impl<T: ReadFrom> ReadFromSeed<()> for T {
    fn read_from_seed<R: Read>(reader: &mut R, seed: ()) -> Result<Self, Error> {
        let _ = seed;
        Self::read_from(reader)
    }
}

pub trait WriteTo {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

macro_rules! primitive_io {
    ($($t:ty: $len:literal,)*) => {$(
        impl ReadFrom for $t {
            fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
                let mut bytes = [0u8; $len];
                reader.read_exact(&mut bytes)?;
                Ok(<$t>::from_le_bytes(bytes))
            }
        }

        impl WriteTo for $t {
            fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                writer.write_all(&self.to_le_bytes())?;
                Ok(())
            }
        }
    )*};
}

primitive_io! {
    u8: 1,
    u16: 2,
    i32: 4,
    u32: 4,
    f32: 4,
}

impl ReadFrom for usize {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        i32::read_from(reader).and_then(|x| Self::try_from(x).map_err(|_| Error::InvalidSave))
    }
}

impl WriteTo for usize {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        i32::try_from(*self)
            .map_err(|_| Error::InvalidSave)
            .and_then(|x| x.write_to(writer))
    }
}

pub struct Length(pub usize);

impl<T: ReadFrom> ReadFromSeed<Length> for Vec<T> {
    fn read_from_seed<R: Read>(reader: &mut R, seed: Length) -> Result<Self, Error> {
        repeat_with(|| T::read_from(reader)).take(seed.0).collect()
    }
}

impl<T: ReadFrom + Default + Copy, const N: usize> ReadFrom for [T; N] {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut arr = [Default::default(); N];
        for slot in &mut arr {
            *slot = T::read_from(reader)?;
        }
        Ok(arr)
    }
}

impl<T: WriteTo> WriteTo for [T] {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for x in self {
            x.write_to(writer)?;
        }
        Ok(())
    }
}

impl ReadFrom for String {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let string_len = usize::read_from(reader)?;
        let mut bytes = vec![0u8; string_len];
        reader.read_exact(&mut bytes)?;
        String::from_utf8(bytes).map_err(|_| Error::InvalidSave)
    }
}

impl WriteTo for str {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.len().write_to(writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

pub fn read_magic<R: Read>(reader: &mut R, magic_bytes: &[u8]) -> Result<(), Error> {
    let mut header = [0u8; 16];
    reader.read_exact(&mut header)?;
    if header != *magic_bytes {
        return Err(Error::InvalidSave);
    }
    Ok(())
}
