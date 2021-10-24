#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum SaveType {
    World,
    Subassembly,
}

#[derive(Debug)]
struct ModInfo {
    mod_id: String,
    mod_version: [i32; 4],
}

#[derive(Debug)]
struct ComponentType {
    numeric_id: u16,
    text_id: String,
}

#[derive(Debug)]
struct Input {
    circuit_state_id: i32,
}

#[derive(Debug)]
struct Output {
    circuit_state_id: i32,
}

#[derive(Debug)]
struct Component {
    address: u32,
    parent: u32,
    type_id: u16,
    position: [f32; 3],
    rotation: [f32; 4],
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    custom_data: Option<Vec<u8>>,
}

#[derive(Debug)]
struct PegAddress {
    is_input: bool,
    component_address: u32,
    peg_index: i32,
}

#[derive(Debug)]
struct Wire {
    start_peg: PegAddress,
    end_peg: PegAddress,
    circuit_state_id: i32,
    rotation: f32,
}

#[derive(Debug)]
enum CircuitStates {
    WorldFormat { circuit_states: Vec<u8> },
    SubassemblyFormat { on_states: Vec<i32> },
}

#[derive(Debug)]
pub struct BlotterFile {
    game_version: [i32; 4],
    save_type: SaveType,
    mods: Vec<ModInfo>,
    component_types: Vec<ComponentType>,
    components: Vec<Component>,
    wires: Vec<Wire>,
    circuit_states: CircuitStates,
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

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
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
                return Err(Error::UnknownSaveType(save_type_id));
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
            game_version,
            save_type,
            mods,
            component_types,
            components,
            wires,
            circuit_states,
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    IoError(std::io::Error),
    InvalidSave,
    IncompatibleVersion(u8),
    UnknownSaveType(u8),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

fn read_u8<R>(reader: &mut R) -> Result<u8, Error>
where
    R: std::io::Read,
{
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(u8::from_le_bytes(bytes))
}

fn read_u16<R>(reader: &mut R) -> Result<u16, Error>
where
    R: std::io::Read,
{
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_i32<R>(reader: &mut R) -> Result<i32, Error>
where
    R: std::io::Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_u32<R>(reader: &mut R) -> Result<u32, Error>
where
    R: std::io::Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_f32<R>(reader: &mut R) -> Result<f32, Error>
where
    R: std::io::Read,
{
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_le_bytes(bytes))
}

fn read_usize<R>(reader: &mut R) -> Result<usize, Error>
where
    R: std::io::Read,
{
    read_i32(reader)?.try_into().map_err(|_| Error::InvalidSave)
}

fn read_string<R>(reader: &mut R) -> Result<String, Error>
where
    R: std::io::Read,
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
    R: std::io::Read,
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
    R: std::io::Read,
{
    read_array(|| read_i32(reader))
}

fn read_bytevec<R>(reader: &mut R, len: usize) -> Result<Vec<u8>, Error>
where
    R: std::io::Read,
{
    let mut vec = vec![0u8; len];
    reader.read_exact(&mut vec)?;
    Ok(vec)
}

fn read_peg_address<R>(reader: &mut R) -> Result<PegAddress, Error>
where
    R: std::io::Read,
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
