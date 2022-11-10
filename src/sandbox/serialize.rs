//! Conversions between the Sandbox editor and Blotter serialization types.

use bitvec::vec::BitVec;

use crate::latest as blotter;
use std::collections::{HashMap, HashSet};

/// The current game version that this save is compatible with.
const GAME_VERSION: [i32; 4] = [0, 91, 0, 510];

/// Serialization context, mainly tracks ID mappings
struct Serializer {
    next_component_id: u32,
    component_id_map: HashMap<super::ComponentId, u32>,
}

impl Serializer {
    fn new() -> Self {
        Self {
            next_component_id: 1,
            component_id_map: HashMap::new(),
        }
    }

    fn register_component(&mut self, id: super::ComponentId) -> u32 {
        let raw_id = self.next_component_id;
        self.next_component_id += 1;
        self.component_id_map.insert(id, raw_id);
        raw_id
    }

    fn get_component(&self, id: super::ComponentId) -> u32 {
        self.component_id_map[&id]
    }

    fn serialize_component(
        &mut self,
        id: super::ComponentId,
        info: &super::ComponentInfo,
    ) -> blotter::Component {
        blotter::Component {
            address: self.register_component(id),
            parent: info.parent.map(|id| self.get_component(id)).unwrap_or(0),
            type_id: info.type_id,
            position: info.position,
            rotation: info.rotation,
            inputs: info
                .inputs
                .iter()
                .map(|peg| self.serialize_input(peg))
                .collect(),
            outputs: info
                .outputs
                .iter()
                .map(|peg| self.serialize_output(peg))
                .collect(),
            custom_data: info.custom_data.clone(),
        }
    }

    fn serialize_input(&self, peg: &super::PegInfo) -> blotter::Input {
        blotter::Input {
            circuit_state_id: peg.net_id.into_raw(),
        }
    }

    fn serialize_output(&self, peg: &super::PegInfo) -> blotter::Output {
        blotter::Output {
            circuit_state_id: peg.net_id.into_raw(),
        }
    }

    fn serialize_wire(&self, wire: &super::WireInfo) -> blotter::Wire {
        blotter::Wire {
            start_peg: self.serialize_peg_address(&wire.a),
            end_peg: self.serialize_peg_address(&wire.b),
            circuit_state_id: wire.net_id.into_raw(),
            rotation: wire.rotation,
        }
    }

    fn serialize_peg_address(&self, addr: &super::PegAddress) -> blotter::PegAddress {
        blotter::PegAddress {
            peg_type: addr.peg_type.into(),
            component_address: self.get_component(addr.component),
            peg_index: addr.peg_index.try_into().unwrap(),
        }
    }
}

impl From<&super::Sandbox> for blotter::BlotterFile {
    fn from(sandbox: &super::Sandbox) -> Self {
        let mut ser = Serializer::new();

        // Blotter format requires that parents must be serialized before children.
        // Serialize components with depth-first, pre-order traversal.
        let mut components = Vec::new();
        let mut stack: Vec<super::ComponentId> = Vec::new();
        stack.extend(&sandbox.root_components);
        while let Some(component_id) = stack.pop() {
            let component = sandbox.components.get(component_id.0).unwrap();
            stack.extend(&component.children);
            components.push(ser.serialize_component(component_id, component))
        }

        let mut states = sandbox.net_states.clone();
        states.set_uninitialized(false);

        Self {
            game_version: GAME_VERSION,
            save_type: blotter::SaveType::World,
            mods: sandbox.mods.clone(),
            component_types: sandbox
                .component_types
                .iter()
                .map(|(name, num)| blotter::ComponentType {
                    numeric_id: *num,
                    text_id: name.clone(),
                })
                .collect(),
            components,
            wires: sandbox
                .wires
                .iter()
                .map(|(_id, wire)| ser.serialize_wire(wire))
                .collect(),
            circuit_states: blotter::CircuitStates::WorldFormat {
                circuit_states: states.into_vec(),
            },
        }
    }
}

/// Deserialization context, mainly tracks ID mappings
struct Deserializer {
    component_id_map: HashMap<u32, super::ComponentId>,
}

impl Deserializer {
    fn new() -> Self {
        Self {
            component_id_map: HashMap::new(),
        }
    }

    fn register_component(&mut self, raw_id: u32, id: super::ComponentId) {
        self.component_id_map.insert(raw_id, id);
    }

    fn get_component(&self, id: u32) -> super::ComponentId {
        self.component_id_map[&id]
    }

    fn deserialize_component(&self, component: &blotter::Component) -> super::ComponentInfo {
        super::ComponentInfo {
            type_id: component.type_id,
            parent: if component.parent == 0 {
                None
            } else {
                Some(self.get_component(component.parent))
            },
            position: component.position,
            rotation: component.rotation,
            children: HashSet::new(),
            inputs: component
                .inputs
                .iter()
                .map(|input| self.deserialize_input(input))
                .collect(),
            outputs: component
                .outputs
                .iter()
                .map(|output| self.deserialize_output(output))
                .collect(),
            custom_data: component.custom_data.clone(),
        }
    }

    fn deserialize_input(&self, input: &blotter::Input) -> super::PegInfo {
        super::PegInfo {
            net_id: super::NetId::from_raw(input.circuit_state_id),
            wires: HashSet::new(),
        }
    }

    fn deserialize_output(&self, output: &blotter::Output) -> super::PegInfo {
        super::PegInfo {
            net_id: super::NetId::from_raw(output.circuit_state_id),
            wires: HashSet::new(),
        }
    }

    fn deserialize_wire(&self, wire: &blotter::Wire) -> super::WireInfo {
        super::WireInfo {
            a: self.deserialize_peg_address(&wire.start_peg),
            b: self.deserialize_peg_address(&wire.end_peg),
            net_id: super::NetId::from_raw(wire.circuit_state_id),
            rotation: wire.rotation,
        }
    }

    fn deserialize_peg_address(&self, addr: &blotter::PegAddress) -> super::PegAddress {
        super::PegAddress {
            peg_type: addr.peg_type.into(),
            component: self.get_component(addr.component_address),
            peg_index: addr.peg_index.try_into().unwrap(),
        }
    }
}

impl From<&blotter::BlotterFile> for super::Sandbox {
    fn from(file: &blotter::BlotterFile) -> Self {
        let mut de = Deserializer::new();

        // Instead of building the sandbox and all the internal cross-references
        // from scratch, re-use the sandbox API as much as possible when loading
        // so there is just one implementation of the cross-referencing.
        let mut sandbox = super::Sandbox::with_meta_info(
            file.component_types
                .iter()
                .map(|ctype| (ctype.text_id.clone(), ctype.numeric_id))
                .collect(),
            file.mods.clone(),
        );

        match &file.circuit_states {
            blotter::CircuitStates::WorldFormat { circuit_states } => {
                sandbox.net_states = BitVec::from_slice(circuit_states);
                for _ in 0..8 * circuit_states.len() {
                    sandbox.nets.insert(super::NetInfo {
                        wires: HashSet::new(),
                        pegs: HashSet::new(),
                    });
                }
            }
            _ => panic!("unsupported circuit state format"),
        }

        for component in &file.components {
            let id = sandbox.insert_component(de.deserialize_component(component));
            de.register_component(component.address, id);
        }
        for wire in &file.wires {
            // TODO bubble error
            let info = de.deserialize_wire(wire);
            sandbox
                .insert_wire(info.a, info.b, info.rotation, Some(info.net_id))
                .unwrap();
        }

        sandbox
    }
}

impl From<super::PegType> for blotter::PegType {
    fn from(peg_type: super::PegType) -> Self {
        match peg_type {
            super::PegType::Input => Self::Input,
            super::PegType::Output => Self::Output,
        }
    }
}

impl From<blotter::PegType> for super::PegType {
    fn from(peg_type: blotter::PegType) -> Self {
        match peg_type {
            blotter::PegType::Input => Self::Input,
            blotter::PegType::Output => Self::Output,
        }
    }
}
