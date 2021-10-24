use std::io::{Read, Write};

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
    pub position: [f32; 3],
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
    pub save_version: u8,
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
            save_version: 5,
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

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        read_magic(reader, b"Logic World save")?;

        let save_version = read_u8(reader)?;
        if save_version != 5 {
            return Err(Error::IncompatibleVersion(save_version));
        }

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
            let position = read_array(|| read_f32(reader))?;
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
            save_version,
            game_version,
            save_type,
            mods,
            component_types,
            components,
            wires,
            circuit_states,
        })
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        writer.write_all(b"Logic World save")?;

        write_u8(self.save_version, writer)?;
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
                write_f32(v, writer)?;
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

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    IoError(std::io::Error),
    InvalidSave,
    IncompatibleVersion(u8),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

fn read_u8<R>(reader: &mut R) -> Result<u8, Error>
where
    R: Read,
{
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(u8::from_le_bytes(bytes))
}

fn read_u16<R>(reader: &mut R) -> Result<u16, Error>
where
    R: Read,
{
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_i32<R>(reader: &mut R) -> Result<i32, Error>
where
    R: Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_u32<R>(reader: &mut R) -> Result<u32, Error>
where
    R: Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_f32<R>(reader: &mut R) -> Result<f32, Error>
where
    R: Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_le_bytes(bytes))
}

fn read_usize<R>(reader: &mut R) -> Result<usize, Error>
where
    R: Read,
{
    read_i32(reader)?.try_into().map_err(|_| Error::InvalidSave)
}

fn read_string<R>(reader: &mut R) -> Result<String, Error>
where
    R: Read,
{
    let string_len = read_usize(reader)?;
    let bytes = read_bytevec(reader, string_len)?;
    String::from_utf8(bytes).map_err(|_| Error::InvalidSave)
}

fn read_vec<F, T>(len: usize, mut reader: F) -> Result<Vec<T>, Error>
where
    F: FnMut() -> Result<T, Error>,
{
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(reader()?);
    }
    Ok(vec)
}

fn read_array<F, T, const LEN: usize>(mut reader: F) -> Result<[T; LEN], Error>
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

fn read_magic<R>(reader: &mut R, magic_bytes: &[u8]) -> Result<(), Error>
where
    R: Read,
{
    let mut header = [0u8; 16];
    reader.read_exact(&mut header)?;
    if header != *magic_bytes {
        return Err(Error::InvalidSave);
    }
    Ok(())
}

fn read_version<R>(reader: &mut R) -> Result<[i32; 4], Error>
where
    R: Read,
{
    read_array(|| read_i32(reader))
}

fn read_bytevec<R>(reader: &mut R, len: usize) -> Result<Vec<u8>, Error>
where
    R: Read,
{
    let mut vec = vec![0u8; len];
    reader.read_exact(&mut vec)?;
    Ok(vec)
}

fn read_peg_address<R>(reader: &mut R) -> Result<PegAddress, Error>
where
    R: Read,
{
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

fn write_u8<W>(v: u8, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn write_u16<W>(v: u16, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn write_i32<W>(v: i32, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn write_u32<W>(v: u32, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn write_f32<W>(v: f32, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    writer.write_all(&v.to_le_bytes())?;
    Ok(())
}

fn write_usize<W>(v: usize, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    let v: i32 = v.try_into().map_err(|_| Error::InvalidSave)?;
    write_i32(v, writer)
}

fn write_str<W>(s: &str, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    write_usize(s.len(), writer)?;
    writer.write_all(s.as_bytes())?;
    Ok(())
}

fn write_peg_address<W>(address: &PegAddress, writer: &mut W) -> Result<(), Error>
where
    W: Write,
{
    let peg_type_id = match address.is_input {
        false => 0,
        true => 1,
    };
    write_u8(peg_type_id, writer)?;
    write_u32(address.component_address, writer)?;
    write_i32(address.peg_index, writer)?;
    Ok(())
}
