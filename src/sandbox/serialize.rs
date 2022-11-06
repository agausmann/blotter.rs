//! Conversions between the Sandbox editor and Blotter serialization types.

use crate::latest as blotter;

/// The current game version that this save is compatible with.
const GAME_VERSION: [i32; 4] = [0, 91, 0, 510];

impl From<&super::Sandbox> for blotter::BlotterFile {
    fn from(sandbox: &super::Sandbox) -> Self {
        // Blotter format requires that parents must be serialized before children.
        // TODO Serialize components with depth-first, pre-order traversal.
        let mut components = Vec::new();
        let mut stack = Vec::new();
        stack.extend(&sandbox.root_components);
        while let Some(component_id) = stack.pop() {
            let component = &sandbox.components[&component_id];
            stack.extend(&component.children);
            components.push((component_id, component).into())
        }

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
                .map(|(_id, wire)| wire.into())
                .collect(),
            circuit_states: blotter::CircuitStates::WorldFormat {
                circuit_states: vec![], //TODO
            },
        }
    }
}

impl From<(super::ComponentId, &super::ComponentInfo)> for blotter::Component {
    fn from((id, component): (super::ComponentId, &super::ComponentInfo)) -> Self {
        Self {
            address: id.into_raw(),
            parent: component.parent.map(|id| id.into_raw()).unwrap_or(0),
            type_id: component.type_id,
            position: component.position,
            rotation: component.rotation,
            inputs: component.inputs.iter().map(|peg| peg.into()).collect(),
            outputs: component.outputs.iter().map(|peg| peg.into()).collect(),
            custom_data: component.custom_data.clone(),
        }
    }
}

impl From<&super::PegInfo> for blotter::Input {
    fn from(peg: &super::PegInfo) -> Self {
        Self {
            circuit_state_id: peg.cluster_id.into_raw(),
        }
    }
}

impl From<&super::PegInfo> for blotter::Output {
    fn from(peg: &super::PegInfo) -> Self {
        Self {
            circuit_state_id: peg.cluster_id.into_raw(),
        }
    }
}

impl From<&super::WireInfo> for blotter::Wire {
    fn from(wire: &super::WireInfo) -> Self {
        Self {
            start_peg: wire.a.into(),
            end_peg: wire.b.into(),
            circuit_state_id: wire.cluster_id.into_raw(),
            rotation: wire.rotation,
        }
    }
}

impl From<super::PegAddress> for blotter::PegAddress {
    fn from(addr: super::PegAddress) -> Self {
        Self {
            peg_type: addr.peg_type.into(),
            component_address: addr.component.into_raw(),
            peg_index: addr.peg_index.try_into().unwrap(),
        }
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
