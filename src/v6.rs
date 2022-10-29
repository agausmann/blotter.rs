use crate::error::Error;
use crate::io::*;
use std::io::{Read, Write};

pub const SAVE_VERSION: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveType {
    World,
    Subassembly,
}

#[derive(Debug)]
pub struct ModInfo {
    pub mod_id: String,
    pub mod_version: [i32; 4],
}

#[derive(Debug)]
pub struct ComponentType {
    pub numeric_id: u16,
    pub text_id: String,
}

#[derive(Debug)]
pub struct Input {
    pub circuit_state_id: i32,
}

#[derive(Debug)]
pub struct Output {
    pub circuit_state_id: i32,
}

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

#[derive(Debug)]
pub struct PegAddress {
    pub is_input: bool,
    pub component_address: u32,
    pub peg_index: i32,
}

#[derive(Debug)]
pub struct Wire {
    pub start_peg: PegAddress,
    pub end_peg: PegAddress,
    pub circuit_state_id: i32,
    pub rotation: f32,
}

#[derive(Debug)]
pub enum CircuitStates {
    WorldFormat { circuit_states: Vec<u8> },
    SubassemblyFormat { on_states: Vec<i32> },
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
        read_magic(reader, b"Logic World save")?;

        let save_version = read_u8(reader)?;
        if save_version != SAVE_VERSION {
            return Err(Error::IncompatibleVersion(save_version));
        }
        Self::read_after_save_version(reader)
    }

    pub(crate) fn read_after_save_version<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let game_version = read_version(reader)?;

        let save_type_id = read_u8(reader)?;
        let save_type = match save_type_id {
            1 => SaveType::World,
            2 => SaveType::Subassembly,
            _ => {
                return Err(Error::InvalidSave);
            }
        };

        let num_components = read_usize(reader)?;
        let num_wires = read_usize(reader)?;

        let num_mods = read_usize(reader)?;
        let mods = read_vec(num_mods, || {
            let mod_id = read_string(reader)?;
            let mod_version = read_version(reader)?;
            Ok(ModInfo {
                mod_id,
                mod_version,
            })
        })?;

        let num_component_types: usize = read_usize(reader)?;
        let component_types = read_vec(num_component_types, || {
            let numeric_id = read_u16(reader)?;
            let text_id = read_string(reader)?;
            Ok(ComponentType {
                numeric_id,
                text_id,
            })
        })?;

        let components = read_vec(num_components, || {
            let address = read_u32(reader)?;
            let parent = read_u32(reader)?;
            let type_id = read_u16(reader)?;
            let position = read_array(|| read_i32(reader))?;
            let rotation = read_array(|| read_f32(reader))?;

            let num_inputs = read_usize(reader)?;
            let inputs = read_vec(num_inputs, || {
                let circuit_state_id = read_i32(reader)?;
                Ok(Input { circuit_state_id })
            })?;

            let num_outputs = read_usize(reader)?;
            let outputs = read_vec(num_outputs, || {
                let circuit_state_id = read_i32(reader)?;
                Ok(Output { circuit_state_id })
            })?;

            let custom_data_len = read_i32(reader)?;
            let custom_data = if custom_data_len < 0 {
                None
            } else {
                Some(read_bytevec(reader, custom_data_len as usize)?)
            };

            Ok(Component {
                address,
                parent,
                type_id,
                position,
                rotation,
                inputs,
                outputs,
                custom_data,
            })
        })?;

        let wires = read_vec(num_wires, || {
            let start_peg = read_peg_address(reader)?;
            let end_peg = read_peg_address(reader)?;
            let circuit_state_id = read_i32(reader)?;
            let rotation = read_f32(reader)?;

            Ok(Wire {
                start_peg,
                end_peg,
                circuit_state_id,
                rotation,
            })
        })?;

        let circuit_states = match save_type {
            SaveType::World => {
                let num_bytes = read_usize(reader)?;
                let circuit_states = read_bytevec(reader, num_bytes)?;
                CircuitStates::WorldFormat { circuit_states }
            }
            SaveType::Subassembly => {
                let num_on_states = read_usize(reader)?;
                let on_states = read_vec(num_on_states, || read_i32(reader))?;
                CircuitStates::SubassemblyFormat { on_states }
            }
        };

        read_magic(reader, b"redstone sux lol")?;

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
        writer.write_all(b"Logic World save")?;

        write_u8(SAVE_VERSION, writer)?;
        for &v in &self.game_version {
            write_i32(v, writer)?;
        }
        let save_type_id = match self.save_type {
            SaveType::World => 1,
            SaveType::Subassembly => 2,
        };
        write_u8(save_type_id, writer)?;

        write_usize(self.components.len(), writer)?;
        write_usize(self.wires.len(), writer)?;

        write_usize(self.mods.len(), writer)?;
        for mod_info in &self.mods {
            write_str(&mod_info.mod_id, writer)?;
            for &v in &mod_info.mod_version {
                write_i32(v, writer)?;
            }
        }

        write_usize(self.component_types.len(), writer)?;
        for component_type in &self.component_types {
            write_u16(component_type.numeric_id, writer)?;
            write_str(&component_type.text_id, writer)?;
        }

        for component in &self.components {
            write_u32(component.address, writer)?;
            write_u32(component.parent, writer)?;
            write_u16(component.type_id, writer)?;
            for &v in &component.position {
                write_i32(v, writer)?;
            }
            for &v in &component.rotation {
                write_f32(v, writer)?;
            }
            write_usize(component.inputs.len(), writer)?;
            for input in &component.inputs {
                write_i32(input.circuit_state_id, writer)?;
            }
            write_usize(component.outputs.len(), writer)?;
            for output in &component.outputs {
                write_i32(output.circuit_state_id, writer)?;
            }
            if let Some(bytes) = &component.custom_data {
                write_usize(bytes.len(), writer)?;
                writer.write_all(&bytes)?;
            } else {
                write_i32(-1, writer)?;
            }
        }

        for wire in &self.wires {
            write_peg_address(&wire.start_peg, writer)?;
            write_peg_address(&wire.end_peg, writer)?;
            write_i32(wire.circuit_state_id, writer)?;
            write_f32(wire.rotation, writer)?;
        }

        match (self.save_type, &self.circuit_states) {
            (SaveType::World, CircuitStates::WorldFormat { circuit_states }) => {
                write_usize(circuit_states.len(), writer)?;
                writer.write_all(&circuit_states)?;
            }
            (SaveType::Subassembly, CircuitStates::SubassemblyFormat { on_states }) => {
                write_usize(on_states.len(), writer)?;
                for &v in on_states {
                    write_i32(v, writer)?;
                }
            }
            _ => {
                return Err(Error::InvalidSave);
            }
        }

        writer.write_all(b"redstone sux lol")?;
        Ok(())
    }
}

fn read_peg_address<R: Read>(reader: &mut R) -> Result<PegAddress, Error> {
    let peg_kind = read_u8(reader)?;
    let is_input = match peg_kind {
        0 => false,
        1 => true,
        _ => return Err(Error::InvalidSave),
    };
    let component_address = read_u32(reader)?;
    let peg_index = read_i32(reader)?;
    Ok(PegAddress {
        is_input,
        component_address,
        peg_index,
    })
}

fn write_peg_address<W: Write>(address: &PegAddress, writer: &mut W) -> Result<(), Error> {
    let peg_type_id = match address.is_input {
        false => 0,
        true => 1,
    };
    write_u8(peg_type_id, writer)?;
    write_u32(address.component_address, writer)?;
    write_i32(address.peg_index, writer)?;
    Ok(())
}
