//! Conversions between the Sandbox editor and Blotter serialization types.

use crate::latest as blotter;
use std::collections::HashSet;

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

impl From<&blotter::BlotterFile> for super::Sandbox {
    fn from(file: &blotter::BlotterFile) -> Self {
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
        for component in &file.components {
            let (id, info) = component.into();
            sandbox.insert_component(id, info);
        }
        for wire in &file.wires {
            // TODO bubble error
            sandbox
                .insert_wire(
                    wire.start_peg.into(),
                    wire.end_peg.into(),
                    wire.rotation,
                    Some(super::ClusterId::from_raw(wire.circuit_state_id)),
                )
                .unwrap();
        }

        //TODO load circuit states

        sandbox
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

impl From<&blotter::Component> for (super::ComponentId, super::ComponentInfo) {
    fn from(component: &blotter::Component) -> Self {
        let id = super::ComponentId::from_raw(component.address).unwrap();
        let info = super::ComponentInfo {
            type_id: component.type_id,
            parent: super::ComponentId::from_raw(component.parent),
            position: component.position,
            rotation: component.rotation,
            children: HashSet::new(),
            inputs: component.inputs.iter().map(|input| input.into()).collect(),
            outputs: component
                .outputs
                .iter()
                .map(|output| output.into())
                .collect(),
            custom_data: component.custom_data.clone(),
        };
        (id, info)
    }
}

impl From<&super::PegInfo> for blotter::Input {
    fn from(peg: &super::PegInfo) -> Self {
        Self {
            circuit_state_id: peg.cluster_id.into_raw(),
        }
    }
}

impl From<&blotter::Input> for super::PegInfo {
    fn from(input: &blotter::Input) -> Self {
        Self {
            cluster_id: super::ClusterId::from_raw(input.circuit_state_id),
            wires: HashSet::new(),
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

impl From<&blotter::Output> for super::PegInfo {
    fn from(input: &blotter::Output) -> Self {
        Self {
            cluster_id: super::ClusterId::from_raw(input.circuit_state_id),
            wires: HashSet::new(),
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

impl From<blotter::PegAddress> for super::PegAddress {
    fn from(addr: blotter::PegAddress) -> Self {
        //TODO bubble errors
        Self {
            component: super::ComponentId::from_raw(addr.component_address).unwrap(),
            peg_type: addr.peg_type.into(),
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

impl From<blotter::PegType> for super::PegType {
    fn from(peg_type: blotter::PegType) -> Self {
        match peg_type {
            blotter::PegType::Input => Self::Input,
            blotter::PegType::Output => Self::Output,
        }
    }
}
