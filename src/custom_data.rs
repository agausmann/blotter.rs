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
        let color = read_array(|| read_u8(reader))?;
        let size_x = read_u32(reader)?;
        let size_z = read_u32(reader)?;
        Ok(Self {
            color,
            size_x,
            size_z,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for x in self.color {
            write_u8(x, writer)?;
        }
        write_u32(self.size_x, writer)?;
        write_u32(self.size_z, writer)?;
        Ok(())
    }
}
