//! Parsers and serializers for vanilla component data.
//!
//! These are designed for Logic World 0.91.0 Preview 510 and may not work for
//! other game versions.

use std::io::{Read, Write};

use crate::error::Error;
use crate::io::*;

pub trait ComponentData: Sized {
    const TYPE_STRING: &'static str;

    fn read<R: Read>(reader: &mut R) -> Result<Self, Error>;

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct CircuitBoard {
    pub color: [u8; 3],
    pub size_x: u32,
    pub size_z: u32,
}

impl ComponentData for CircuitBoard {
    const TYPE_STRING: &'static str = "MHG.CircuitBoard";

    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let color = ReadFrom::read_from(reader)?;
        let size_x = ReadFrom::read_from(reader)?;
        let size_z = ReadFrom::read_from(reader)?;
        Ok(Self {
            color,
            size_x,
            size_z,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.color.write_to(writer)?;
        self.size_x.write_to(writer)?;
        self.size_z.write_to(writer)?;
        Ok(())
    }
}
