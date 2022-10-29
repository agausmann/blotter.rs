use crate::{v5, v6};

impl From<v5::BlotterFile> for v6::BlotterFile {
    fn from(file: v5::BlotterFile) -> Self {
        Self {
            game_version: file.game_version,
            save_type: file.save_type,
            mods: file.mods,
            component_types: file.component_types,
            components: file.components.into_iter().map(Into::into).collect(),
            wires: file.wires,
            circuit_states: file.circuit_states,
        }
    }
}

impl From<v5::Component> for v6::Component {
    fn from(component: v5::Component) -> Self {
        Self {
            address: component.address,
            parent: component.parent,
            type_id: component.type_id,
            position: floating_to_fixed_position(component.position),
            rotation: component.rotation,
            inputs: component.inputs,
            outputs: component.outputs,
            custom_data: component.custom_data,
        }
    }
}

fn floating_to_fixed_position([x, y, z]: [f32; 3]) -> [i32; 3] {
    const CONVERSION_FACTOR: f32 = 1000.0;
    [
        (x * CONVERSION_FACTOR) as i32,
        (y * CONVERSION_FACTOR) as i32,
        (z * CONVERSION_FACTOR) as i32,
    ]
}
