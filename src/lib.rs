pub mod error;
pub(crate) mod io;
pub mod v5;

use io::read_u8;
use std::io::{Read, Write};

use crate::error::Error;
use crate::io::read_magic;

#[derive(Debug)]
pub enum BlotterFile {
    V5(v5::BlotterFile),
}

impl BlotterFile {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        read_magic(reader, b"Logic World save")?;
        let save_version = read_u8(reader)?;
        match save_version {
            5 => v5::BlotterFile::read_after_save_version(reader).map(Self::V5),
            _ => Err(Error::IncompatibleVersion(save_version)),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::V5(file) => file.write(writer),
        }
    }
}
