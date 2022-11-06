//! High-level API for constructing and modifying a "Sandbox" / World.

mod serialize;

use crate::latest::ModInfo;
use std::{
    collections::{HashMap, HashSet},
    iter::repeat_with,
    num::NonZeroU32,
};

/// An in-memory representation of a Sandbox that is easy to modify.
pub struct Sandbox {
    next_component: u32,
    next_cluster: u32,
    next_wire: u32,
    next_type: u16,
    root_components: HashSet<ComponentId>,
    components: HashMap<ComponentId, ComponentInfo>,
    wires: HashMap<WireId, WireInfo>,
    clusters: HashMap<ClusterId, ClusterInfo>,
    component_types: HashMap<String, u16>,
    mods: Vec<ModInfo>,
}

impl Sandbox {
    /// Create a new, empty sandbox level.
    pub fn new() -> Self {
        Self::with_meta_info(default_component_types_map(), Vec::new())
    }

    /// Create an empty sandbox level with custom component-type / mod info.
    fn with_meta_info(component_types: HashMap<String, u16>, mods: Vec<ModInfo>) -> Self {
        Self {
            next_component: 1,
            next_cluster: 0,
            next_wire: 0,
            next_type: component_types.values().copied().max().unwrap_or(0),
            root_components: HashSet::new(),
            components: HashMap::new(),
            wires: HashMap::new(),
            clusters: HashMap::new(),
            component_types,
            mods,
        }
    }

    pub fn add_component(&mut self, component: &ComponentBuilder) -> ComponentId {
        let info = ComponentInfo {
            type_id: self.get_component_type(component.id),
            parent: component.parent,
            position: component.position,
            rotation: component.rotation,
            children: HashSet::new(),
            inputs: repeat_with(|| PegInfo {
                cluster_id: self.make_cluster(),
                wires: HashSet::new(),
            })
            .take(component.num_inputs as usize)
            .collect(),
            outputs: repeat_with(|| PegInfo {
                cluster_id: self.make_cluster(),
                wires: HashSet::new(),
            })
            .take(component.num_outputs as usize)
            .collect(),
            custom_data: component.custom_data.clone(),
        };
        let id = ComponentId(NonZeroU32::new(self.next_component).unwrap());
        self.insert_component(id, info);
        id
    }

    /// Internal logic of `add_component` that is shared with savefile loading.
    /// Savefiles store the component and cluster IDs, so they can be directly
    /// passed instead of being allocated (as is done by `add_component`).
    fn insert_component(&mut self, id: ComponentId, info: ComponentInfo) {
        // TODO bubble this error (could be triggered by an invalid savefile).
        assert!(!self.components.contains_key(&id));

        // Update component ID allocator.
        // TODO make this less sparse, 2 billion is not very much to work with
        // over the lifetime of a world.
        self.next_component = self.next_component.max(id.into_raw() + 1);

        // Add parent-child cross-reference.
        if let Some(parent) = info.parent {
            // Valid savefiles will store and load the parent before the child,
            // so it is reasonable to assume the parent exists here.
            //TODO bubble this error; it should be recoverable in file loading.
            self.components
                .get_mut(&parent)
                .unwrap()
                .children
                .insert(id);
        } else {
            self.root_components.insert(id);
        }

        // Add component info.
        self.components.insert(id, info);
    }

    pub fn add_wire(
        &mut self,
        addr_a: PegAddress,
        addr_b: PegAddress,
        rotation: f32,
    ) -> Result<WireId, AddWireError> {
        self.insert_wire(addr_a, addr_b, rotation, None)
    }

    /// Like the `add_component` / `insert_component` duality, this has logic
    /// shared between `add_wire` and savefile loading. Unlike
    /// `insert_component`, this does allocate the wire ID automatically,
    /// because the save file doesn't use wire IDs.
    fn insert_wire(
        &mut self,
        addr_a: PegAddress,
        addr_b: PegAddress,
        rotation: f32,
        cluster_id: Option<ClusterId>,
    ) -> Result<WireId, AddWireError> {
        // It is illegal to directly connect output pegs.
        if addr_a.peg_type == PegType::Output && addr_b.peg_type == PegType::Output {
            return Err(AddWireError::InvalidPegAddress)?;
        }

        let peg_a = self
            .get_peg(&addr_a)
            .ok_or(AddWireError::InvalidPegAddress)?;
        let peg_b = self
            .get_peg(&addr_b)
            .ok_or(AddWireError::InvalidPegAddress)?;
        // If there is already a wire connecting these pegs, nothing needs to be
        // done.
        if let Some(&wire_id) = peg_a.wires.intersection(&peg_b.wires).next() {
            return Ok(wire_id);
        }

        let cluster_id = match cluster_id {
            Some(id) => {
                // If cluster ID is specified (i.e. from an existing savefile),
                // check savefile validity: ensure that both endpoints and the
                // wire all have the same cluster.

                //TODO bubble this error
                assert!(id == peg_a.cluster_id && id == peg_b.cluster_id);

                id
            }
            None => {
                // If no cluster ID is given (i.e. caller is `add_wire`), then
                // obtain one from merging the two endpoints.
                self.merge_clusters(peg_a.cluster_id, peg_b.cluster_id)
            }
        };

        // Create wire.
        let wire_id = WireId(self.next_wire);
        self.next_wire += 1;
        self.wires.insert(
            wire_id,
            WireInfo {
                a: addr_a,
                b: addr_b,
                cluster_id,
                rotation,
            },
        );

        // Register wire references in pegs and cluster.
        self.get_peg_mut(&addr_a).unwrap().wires.insert(wire_id);
        self.get_peg_mut(&addr_b).unwrap().wires.insert(wire_id);
        self.clusters
            .get_mut(&cluster_id)
            .unwrap()
            .wires
            .insert(wire_id);

        Ok(wire_id)
    }

    pub fn remove_component(&mut self, id: ComponentId) {
        // TODO Remove all wires connected to this component.

        // TODO Remove component.

        // TODO Remove peg-cluster cross-references. Remove clusters if empty.

        todo!()
    }

    pub fn remove_wire(&mut self, id: WireId) {
        // TODO Remove wire-cluster cross-reference. Remove cluster if empty.

        // TODO Remove wire.

        // TODO Split cluster if necessary.

        todo!()
    }

    fn get_component_type(&mut self, id: &str) -> u16 {
        match self.component_types.get(id) {
            Some(&x) => x,
            None => {
                let num = self.next_type;
                self.next_type += 1;
                self.component_types.insert(id.to_owned(), num);
                num
            }
        }
    }

    fn make_cluster(&mut self) -> ClusterId {
        let id = ClusterId(self.next_cluster);
        self.next_cluster += 1;
        id
    }

    fn merge_clusters(&mut self, id_a: ClusterId, id_b: ClusterId) -> ClusterId {
        // Nothing needs to be done if the two clusters are the same.
        if id_a == id_b {
            return id_a;
        }

        let a = &self.clusters[&id_a];
        let b = &self.clusters[&id_b];
        // Merge the smaller cluster into the larger cluster.
        let (id_dest, id_src) = if a.size() >= b.size() {
            (id_a, id_b)
        } else {
            (id_b, id_a)
        };

        let src = self.clusters.remove(&id_src).unwrap();
        // Update cluster cross-references:
        for wire_id in &src.wires {
            self.wires.get_mut(wire_id).unwrap().cluster_id = id_dest;
        }
        for peg_id in &src.pegs {
            self.get_peg_mut(peg_id).unwrap().cluster_id = id_dest;
        }

        // Move references from src into dest:
        let dest = self.clusters.get_mut(&id_dest).unwrap();
        dest.pegs.extend(src.pegs);
        dest.wires.extend(src.wires);

        id_dest
    }

    fn get_peg(&self, addr: &PegAddress) -> Option<&PegInfo> {
        self.components
            .get(&addr.component)
            .and_then(|component| component.get_peg(addr))
    }

    fn get_peg_mut(&mut self, addr: &PegAddress) -> Option<&mut PegInfo> {
        self.components
            .get_mut(&addr.component)
            .and_then(|component| component.get_peg_mut(addr))
    }
}

#[derive(Clone)]
pub struct ComponentBuilder<'a> {
    id: &'a str,
    parent: Option<ComponentId>,
    position: [i32; 3],
    rotation: [f32; 4],
    num_inputs: u32,
    num_outputs: u32,
    custom_data: Option<Vec<u8>>,
}

impl<'a> ComponentBuilder<'a> {
    pub fn new(id: &'a str) -> Self {
        Self {
            id,
            parent: None,
            position: [0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            num_inputs: 0,
            num_outputs: 0,
            custom_data: None,
        }
    }

    pub fn parent(self, parent: Option<ComponentId>) -> Self {
        Self { parent, ..self }
    }

    pub fn position(self, position: [i32; 3]) -> Self {
        Self { position, ..self }
    }

    pub fn rotation(self, rotation: [f32; 4]) -> Self {
        Self { rotation, ..self }
    }

    pub fn num_inputs(self, num_inputs: u32) -> Self {
        Self { num_inputs, ..self }
    }

    pub fn num_outputs(self, num_outputs: u32) -> Self {
        Self {
            num_outputs,
            ..self
        }
    }

    pub fn custom_data(self, custom_data: Option<Vec<u8>>) -> Self {
        Self {
            custom_data,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(NonZeroU32);

impl ComponentId {
    fn into_raw(self) -> u32 {
        self.0.into()
    }

    fn from_raw(raw: u32) -> Option<Self> {
        NonZeroU32::new(raw).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClusterId(u32);

impl ClusterId {
    fn into_raw(self) -> i32 {
        self.0.try_into().unwrap()
    }

    fn from_raw(raw: i32) -> Self {
        Self(raw.try_into().unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireId(u32);

#[derive(Debug)]
pub enum AddWireError {
    InvalidPegAddress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PegType {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PegAddress {
    pub component: ComponentId,
    pub peg_type: PegType,
    pub peg_index: usize,
}

struct ComponentInfo {
    type_id: u16,
    parent: Option<ComponentId>,
    position: [i32; 3],
    rotation: [f32; 4],
    children: HashSet<ComponentId>,
    inputs: Vec<PegInfo>,
    outputs: Vec<PegInfo>,
    custom_data: Option<Vec<u8>>,
}

impl ComponentInfo {
    fn get_peg(&self, addr: &PegAddress) -> Option<&PegInfo> {
        match addr.peg_type {
            PegType::Input => self.inputs.get(addr.peg_index),
            PegType::Output => self.outputs.get(addr.peg_index),
        }
    }

    fn get_peg_mut(&mut self, addr: &PegAddress) -> Option<&mut PegInfo> {
        match addr.peg_type {
            PegType::Input => self.inputs.get_mut(addr.peg_index),
            PegType::Output => self.outputs.get_mut(addr.peg_index),
        }
    }
}

struct PegInfo {
    cluster_id: ClusterId,
    wires: HashSet<WireId>,
}

#[derive(Clone, Copy)]
struct WireInfo {
    a: PegAddress,
    b: PegAddress,
    cluster_id: ClusterId,
    rotation: f32,
}

struct ClusterInfo {
    wires: HashSet<WireId>,
    pegs: HashSet<PegAddress>,
}

impl ClusterInfo {
    fn size(&self) -> usize {
        self.wires.len() + self.pegs.len()
    }
}

const DEFAULT_COMPONENT_TYPES: [(u16, &str); 31] = [
    (0, "MHG.Inverter"),
    (1, "MHG.XorGate"),
    (2, "MHG.AndGate"),
    (3, "MHG.Delayer"),
    (4, "MHG.DLatch"),
    (5, "MHG.Randomizer"),
    (6, "MHG.Relay"),
    (7, "MHG.Buffer_WithOutput"),
    (8, "MHG.Buffer"),
    (9, "MHG.CircuitBoard"),
    (10, "MHG.Mount"),
    (11, "MHG.Peg"),
    (12, "MHG.ThroughPeg"),
    (13, "MHG.Socket"),
    (14, "MHG.ThroughSocket"),
    (15, "MHG.ChubbySocket"),
    (16, "MHG.ChubbyThroughSocket"),
    (17, "MHG.Label"),
    (18, "MHG.PanelLabel"),
    (19, "MHG.Chair"),
    (20, "MHG.Flag"),
    (21, "MHG.StandingDisplay"),
    (22, "MHG.PanelDisplay"),
    (23, "MHG.Singer"),
    (24, "MHG.Drum"),
    (25, "MHG.Switch"),
    (26, "MHG.PanelSwitch"),
    (27, "MHG.Button"),
    (28, "MHG.PanelButton"),
    (29, "MHG.Key"),
    (30, "MHG.PanelKey"),
];

fn default_component_types_map() -> HashMap<String, u16> {
    DEFAULT_COMPONENT_TYPES
        .iter()
        .map(|&(num, name)| (name.to_owned(), num))
        .collect()
}
