use crate::error::Error;
use crate::io::*;
use std::io::{Read, Write};

pub const SAVE_VERSION: u8 = 5;
pub const SAVE_HEADER: &[u8; 16] = b"Logic World save";
pub const SAVE_FOOTER: &[u8; 16] = b"redstone sux lol";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveType {
    World,
    Subassembly,
}

impl ReadFrom for SaveType {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let id = u8::read_from(reader)?;
        match id {
            1 => Ok(Self::World),
            2 => Ok(Self::Subassembly),
            _ => Err(Error::InvalidSave),
        }
    }
}

impl WriteTo for SaveType {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let id: u8 = match self {
            Self::World => 1,
            Self::Subassembly => 2,
        };
        id.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModInfo {
    pub mod_id: String,
    pub mod_version: [i32; 4],
}

impl ReadFrom for ModInfo {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mod_id = ReadFrom::read_from(reader)?;
        let mod_version = ReadFrom::read_from(reader)?;
        Ok(Self {
            mod_id,
            mod_version,
        })
    }
}

impl WriteTo for ModInfo {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.mod_id.write_to(writer)?;
        self.mod_version.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ComponentType {
    pub numeric_id: u16,
    pub text_id: String,
}

impl ReadFrom for ComponentType {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let numeric_id = ReadFrom::read_from(reader)?;
        let text_id = ReadFrom::read_from(reader)?;
        Ok(Self {
            numeric_id,
            text_id,
        })
    }
}

impl WriteTo for ComponentType {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.numeric_id.write_to(writer)?;
        self.text_id.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Input {
    pub circuit_state_id: i32,
}

impl ReadFrom for Input {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let circuit_state_id = ReadFrom::read_from(reader)?;
        Ok(Self { circuit_state_id })
    }
}

impl WriteTo for Input {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.circuit_state_id.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Output {
    pub circuit_state_id: i32,
}

impl ReadFrom for Output {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let circuit_state_id = ReadFrom::read_from(reader)?;
        Ok(Self { circuit_state_id })
    }
}

impl WriteTo for Output {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.circuit_state_id.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Component {
    pub address: u32,
    pub parent: u32,
    pub type_id: u16,
    pub position: [f32; 3],
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

#[derive(Debug, Clone, Copy)]
pub enum PegType {
    Output,
    Input,
}

impl ReadFrom for PegType {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let id = u8::read_from(reader)?;
        match id {
            0 => Ok(Self::Output),
            1 => Ok(Self::Input),
            _ => Err(Error::InvalidSave),
        }
    }
}

impl WriteTo for PegType {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let id: u8 = match self {
            Self::Output => 0,
            Self::Input => 1,
        };
        id.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PegAddress {
    pub peg_type: PegType,
    pub component_address: u32,
    pub peg_index: i32,
}

impl ReadFrom for PegAddress {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let peg_type = ReadFrom::read_from(reader)?;
        let component_address = ReadFrom::read_from(reader)?;
        let peg_index = ReadFrom::read_from(reader)?;

        Ok(Self {
            peg_type,
            component_address,
            peg_index,
        })
    }
}

impl WriteTo for PegAddress {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.peg_type.write_to(writer)?;
        self.component_address.write_to(writer)?;
        self.peg_index.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Wire {
    pub start_peg: PegAddress,
    pub end_peg: PegAddress,
    pub circuit_state_id: i32,
    pub rotation: f32,
}

impl ReadFrom for Wire {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let start_peg = ReadFrom::read_from(reader)?;
        let end_peg = ReadFrom::read_from(reader)?;
        let circuit_state_id = ReadFrom::read_from(reader)?;
        let rotation = ReadFrom::read_from(reader)?;
        Ok(Self {
            start_peg,
            end_peg,
            circuit_state_id,
            rotation,
        })
    }
}

impl WriteTo for Wire {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.start_peg.write_to(writer)?;
        self.end_peg.write_to(writer)?;
        self.circuit_state_id.write_to(writer)?;
        self.rotation.write_to(writer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum CircuitStates {
    WorldFormat { circuit_states: Vec<u8> },
    SubassemblyFormat { on_states: Vec<i32> },
}

impl ReadFromSeed<SaveType> for CircuitStates {
    fn read_from_seed<R: Read>(reader: &mut R, seed: SaveType) -> Result<Self, Error> {
        match seed {
            SaveType::World => {
                let num_bytes = usize::read_from(reader)?;
                let mut circuit_states = vec![0u8; num_bytes];
                reader.read_exact(&mut circuit_states)?;
                Ok(Self::WorldFormat { circuit_states })
            }
            SaveType::Subassembly => {
                let num_on_states = usize::read_from(reader)?;
                let on_states = Vec::read_from_seed(reader, Length(num_on_states))?;
                Ok(Self::SubassemblyFormat { on_states })
            }
        }
    }
}

impl WriteTo for CircuitStates {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::WorldFormat { circuit_states } => {
                circuit_states.len().write_to(writer)?;
                writer.write_all(&circuit_states)?;
            }
            Self::SubassemblyFormat { on_states } => {
                on_states.len().write_to(writer)?;
                on_states.write_to(writer)?;
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
