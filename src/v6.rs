use crate::error::Error;
use crate::io::*;
use std::io::{Read, Write};

pub const SAVE_VERSION: u8 = 6;

// Unchanged from previous version:
pub use crate::v5::{
    CircuitStates, ComponentType, Input, ModInfo, Output, PegAddress, PegType, SaveType, Wire,
    SAVE_FOOTER, SAVE_HEADER,
};

#[derive(Debug)]
pub struct Component {
    pub address: u32,
    pub parent: u32,
    pub type_id: u16,
    pub position: [i32; 3],
    pub rotation: [f32; 4],
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub custom_data: Option<Vec<u8>>,
}

impl ReadFrom for Component {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let address = ReadFrom::read_from(reader)?;
        let parent = ReadFrom::read_from(reader)?;
        let type_id = ReadFrom::read_from(reader)?;
        let position = ReadFrom::read_from(reader)?;
        let rotation = ReadFrom::read_from(reader)?;

        let num_inputs = usize::read_from(reader)?;
        let inputs = ReadFromSeed::read_from_seed(reader, Length(num_inputs))?;

        let num_outputs = usize::read_from(reader)?;
        let outputs = ReadFromSeed::read_from_seed(reader, Length(num_outputs))?;

        let custom_data_len = i32::read_from(reader)?;
        let custom_data = if custom_data_len < 0 {
            None
        } else {
            let mut data = vec![0u8; custom_data_len as usize];
            reader.read_exact(&mut data)?;
            Some(data)
        };

        Ok(Self {
            address,
            parent,
            type_id,
            position,
            rotation,
            inputs,
            outputs,
            custom_data,
        })
    }
}

impl WriteTo for Component {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.address.write_to(writer)?;
        self.parent.write_to(writer)?;
        self.type_id.write_to(writer)?;
        self.position.write_to(writer)?;
        self.rotation.write_to(writer)?;

        self.inputs.len().write_to(writer)?;
        self.inputs.write_to(writer)?;

        self.outputs.len().write_to(writer)?;
        self.outputs.write_to(writer)?;

        match &self.custom_data {
            None => {
                (-1_i32).write_to(writer)?;
            }
            Some(data) => {
                data.len().write_to(writer)?;
                writer.write_all(&data)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BlotterFile {
    pub game_version: [i32; 4],
    pub save_type: SaveType,
    pub mods: Vec<ModInfo>,
    pub component_types: Vec<ComponentType>,
    pub components: Vec<Component>,
    pub wires: Vec<Wire>,
    pub circuit_states: CircuitStates,
}

impl BlotterFile {
    pub fn new(game_version: [i32; 4]) -> Self {
        Self {
            game_version,
            save_type: SaveType::World,
            mods: Vec::new(),
            component_types: Vec::new(),
            components: Vec::new(),
            wires: Vec::new(),
            circuit_states: CircuitStates::WorldFormat {
                circuit_states: Vec::new(),
            },
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        read_magic(reader, SAVE_HEADER)?;

        let save_version = u8::read_from(reader)?;
        if save_version != SAVE_VERSION {
            return Err(Error::IncompatibleVersion(save_version));
        }
        Self::read_after_save_version(reader)
    }

    pub(crate) fn read_after_save_version<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let game_version = ReadFrom::read_from(reader)?;

        let save_type = SaveType::read_from(reader)?;
        let num_components = usize::read_from(reader)?;
        let num_wires = usize::read_from(reader)?;

        let num_mods = usize::read_from(reader)?;
        let mods = Vec::read_from_seed(reader, Length(num_mods))?;

        let num_component_types: usize = usize::read_from(reader)?;
        let component_types = Vec::read_from_seed(reader, Length(num_component_types))?;

        let components = Vec::read_from_seed(reader, Length(num_components))?;
        let wires = Vec::read_from_seed(reader, Length(num_wires))?;

        let circuit_states = CircuitStates::read_from_seed(reader, save_type)?;

        read_magic(reader, SAVE_FOOTER)?;

        Ok(Self {
            game_version,
            save_type,
            mods,
            component_types,
            components,
            wires,
            circuit_states,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(SAVE_HEADER)?;

        SAVE_VERSION.write_to(writer)?;
        self.game_version.write_to(writer)?;
        self.save_type.write_to(writer)?;

        self.components.len().write_to(writer)?;
        self.wires.len().write_to(writer)?;

        self.mods.len().write_to(writer)?;
        self.mods.write_to(writer)?;

        self.component_types.len().write_to(writer)?;
        self.component_types.write_to(writer)?;

        self.components.write_to(writer)?;
        self.wires.write_to(writer)?;

        match (self.save_type, &self.circuit_states) {
            (SaveType::World, CircuitStates::WorldFormat { .. }) => {}
            (SaveType::Subassembly, CircuitStates::SubassemblyFormat { .. }) => {}
            _ => {
                return Err(Error::InvalidSave);
            }
        }
        self.circuit_states.write_to(writer)?;

        writer.write_all(SAVE_FOOTER)?;
        Ok(())
    }
}
