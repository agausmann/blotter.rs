use declio::ctx::Len;
use declio::util::{magic_bytes, zero_one, LittleEndian};
use declio::{Decode, Encode};
use std::convert::TryInto;

type ComponentId = LittleEndian<u16>;
type ComponentAddress = LittleEndian<u32>;
type Int = LittleEndian<i32>;
type Float = LittleEndian<f32>;

#[derive(Debug, Clone, Copy, PartialEq, Encode, Decode)]
#[declio(id_type = "u8")]
enum SaveType {
    #[declio(id = "1")]
    World,
    #[declio(id = "2")]
    Subassembly,
}

#[derive(Debug, Encode, Decode)]
pub struct BlotterFile {
    header: Header,
    save_version: u8,
    game_version: [Int; 4],
    save_type: SaveType,
    components_len: Int,
    wires_len: Int,
    component_ids_len: Int,
    #[declio(ctx = "Len(component_ids_len.0.try_into()?)")]
    component_ids: Vec<ComponentIdMapping>,
    #[declio(ctx = "Len(components_len.0.try_into()?)")]
    components: Vec<Component>,
    #[declio(ctx = "Len(wires_len.0.try_into()?)")]
    wires: Vec<Wire>,
    #[declio(ctx = "*save_type")]
    circuit_states: CircuitStates,
    footer: Footer,
}

magic_bytes! {
    #[derive(Debug)]
    Header(b"Logic World save");

    #[derive(Debug)]
    Footer(b"redstone sux lol");
}

#[derive(Debug, Encode, Decode)]
struct Text {
    len: Int,
    #[declio(with = "declio::util::utf8", ctx = "Len(len.0.try_into()?)")]
    value: String,
}

#[derive(Debug, Encode, Decode)]
struct ComponentIdMapping {
    numeric_id: ComponentId,
    text_id: Text,
}

#[derive(Debug, Encode, Decode)]
struct Input {
    #[declio(with = "zero_one")]
    exclusive: bool,
    circuit_state_id: Int,
}

#[derive(Debug, Encode, Decode)]
struct Output {
    circuit_state_id: Int,
}

#[derive(Debug, Encode, Decode)]
struct Component {
    address: ComponentAddress,
    parent: ComponentAddress,
    type_id: ComponentId,
    position: [Float; 3],
    rotation: [Float; 4],
    inputs_len: u8,
    #[declio(ctx = "Len((*inputs_len).try_into()?)")]
    inputs: Vec<Input>,
    outputs_len: u8,
    #[declio(ctx = "Len((*outputs_len).try_into()?)")]
    outputs: Vec<Output>,
    custom_data_len: Int,
    #[declio(ctx = "Len(custom_data_len.0.try_into()?)")]
    custom_data: Vec<u8>,
}

#[derive(Debug, Encode, Decode)]
struct PegAddress {
    #[declio(with = "zero_one")]
    is_input: bool,
    component: ComponentAddress,
    index: u8,
}

#[derive(Debug, Encode, Decode)]
struct Wire {
    peg_1: PegAddress,
    peg_2: PegAddress,
    circuit_state_id: Int,
    rotation: Float,
}

#[derive(Debug, Encode, Decode)]
#[declio(ctx = "save_type: SaveType", id_expr = "save_type")]
enum CircuitStates {
    #[declio(id = "SaveType::World")]
    WorldFormat {
        len: Int,
        #[declio(ctx = "Len(len.0.try_into()?)")]
        circuit_states: Vec<u8>,
    },
    #[declio(id = "SaveType::Subassembly")]
    SubassemblyFormat {
        len: Int,
        #[declio(ctx = "Len(len.0.try_into()?)")]
        on_states: Vec<Int>,
    },
}
