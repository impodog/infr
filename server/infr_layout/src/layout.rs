use crate::InfrError;
use crate::scripts::*;
use mlua::prelude::*;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::{Arc, RwLock},
};

use infr_solver::*;

/// Describes the manner of movement.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Manner {
    /// Swipes towards that position and may push objects.
    Swipe(Direction),
    /// The object is put onto that position without a direction.
    Teleport,
    /// Removes the object. This ignores the dest of the movement.
    Remove,
    /// Adds an object to the dest.
    Add(ObjectDesc),
}

/// Stores movement destination and the manner of movement.
/// Each object on each step can only have one of movement (excluding equal ones).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Movement {
    pub manner: Manner,
    /// The id of the object that executes the movement.
    pub object: u32,
    /// This movement must be performed for this movement to work.
    pub required_by: u32,
    pub dest: Coord,
    /// This movement can never be performed if set to true.
    pub forbid: bool,
}
impl PartialOrd for Movement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Movement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object
            .cmp(&other.object)
            .then_with(|| self.dest.cmp(&other.dest))
    }
}

/// A signal from player input. Lua functions can respond to this accordingly.
pub struct Signal {
    /// The input direction of the player at the start of all movements.
    pub direction: Option<Direction>,
    /// The round number of the signal. For round > 0, it is called after the direct movement.
    pub round: u32,
}

pub type ArcMap = Arc<RwLock<Map>>;

/// Represents the game layout and parses movements.
#[derive(Debug, Clone)]
pub struct Layout {
    pub map: Map,
    move_queue: Vec<Movement>,
    /// The list of objects listening to the key object's movements.
    /// Only listened movements are sent to the listeners.
    listen: HashMap<u32, Vec<u32>>,
    to_index: HashMap<u32, usize>,
    /// The queue for all remove movements. This will be collected at the end of `Self::step`.
    remove_queue: Vec<usize>,
}

impl Layout {
    /// Creates an empty game layout. To add an object, use methods from `self.map`.
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            move_queue: Default::default(),
            listen: Default::default(),
            to_index: Default::default(),
            remove_queue: Default::default(),
        }
    }

    /// Does the chores: initializes map for logic check. Returns all relevant groups.
    ///
    /// You must call `self.map.revert` after this function.
    fn init_map(&mut self) -> Result<BTreeSet<String>, InfrError> {
        self.map.parse();
        self.map.build_solver();
        let relevant_groups = self.map.get_relevant_groups();
        self.map.prove_groups(&relevant_groups);
        self.map.check_contradiction()?;
        self.map.check_overlap()?;
        self.remove_queue.clear();
        Ok(relevant_groups)
    }

    /// Performs the movement that actually changes the map.
    ///
    /// Returns the object id that performs the movement. If the movement is Add, the fresh id is returned.
    ///
    /// If the movement is ill-formed, returns an error.
    fn perform_movement(&mut self, movement: Movement) -> Result<u32, InfrError> {
        match &movement.manner {
            Manner::Add(object_desc) => {
                if object_desc.group.len() == 1 {
                    let mut object = Object::new(
                        movement.dest,
                        object_desc.kind,
                        object_desc.group.iter().next().cloned().unwrap(),
                    );
                    object.direction = object_desc.direction;
                    let id = object.id();
                    self.map.push(object);
                    Ok(id)
                } else {
                    Err(InfrError::IllFormed(movement))
                }
            }
            Manner::Swipe(direction) => {
                let index = *self
                    .to_index
                    .get(&movement.object)
                    .ok_or_else(|| InfrError::NoSuchId(movement.object))?;
                let object = self
                    .map
                    .objects
                    .get_mut(index)
                    .expect("Stored index should be valid");
                object.coord = movement.dest;
                object.direction = Some(*direction);
                Ok(object.id())
            }
            Manner::Teleport => {
                let index = *self
                    .to_index
                    .get(&movement.object)
                    .ok_or_else(|| InfrError::NoSuchId(movement.object))?;
                let object = self
                    .map
                    .objects
                    .get_mut(index)
                    .expect("Stored index should be valid");
                object.coord = movement.dest;
                Ok(object.id())
            }
            Manner::Remove => {
                let index = *self
                    .to_index
                    .get(&movement.object)
                    .ok_or_else(|| InfrError::NoSuchId(movement.object))?;
                self.remove_queue.push(index);
                Ok(movement.object)
            }
        }
    }

    /// Converts current map status to a lua value.
    fn convert_to_lua(&self, lua: &Lua) -> Result<LuaValue, InfrError> {
        let mut objects = BTreeMap::<Coord, Vec<ObjectDesc>>::new();
        for (index, object) in self.map.objects.iter().enumerate() {
            objects.entry(object.coord).or_default().push(
                self.map
                    .get_object_desc(index)
                    .expect("There should be object description at an existing object index."),
            );
        }
        let table = lua.create_table()?;
        for (coord, objects) in objects.into_iter() {
            table.set(coord.to_list(), objects)?;
        }
        Ok(LuaValue::Table(table))
    }

    pub fn perform_listen(
        &self,
        listener: usize,
        layout: &LuaValue,
        movement: &LuaValue,
        _lua: &Lua,
        scripts: &Scripts,
        new_movements: &mut Vec<Movement>,
    ) -> Result<(), InfrError> {
        let groups = self
            .map
            .groups()
            .get(listener)
            .expect("Listener should be valid");
        let coord = self.map.objects[listener].coord;
        for group in groups.iter() {
            if let Some(id) = scripts.feature_names.lock().unwrap().get(group).copied() {
                let features = scripts.features.lock().unwrap();
                let feature = features
                    .get(&id)
                    .expect("Feature name should correspond to a feature implementation");
                if let Some(movements) = feature.respond(layout, movement, coord)? {
                    new_movements.extend(movements.into_iter());
                }
            }
        }
        Ok(())
    }

    /// Step the movements with a external signal. This will also clear any previous record of movement or errors.
    /// Returns all the movements that were performed(may be empty), removing duplicate ones.
    ///
    /// If the signal has a direction, the scripts directly responds to player input, otherwise they will do subsequent moves.
    ///
    /// You should call this repeatedly to parse the subsequent movements.
    pub fn step(
        &mut self,
        signal: Signal,
        lua: &Lua,
        scripts: &Scripts,
    ) -> Result<Vec<Movement>, InfrError> {
        let _relevant_groups = self.init_map()?;
        let layout = self.convert_to_lua(lua)?;
        let signal = signal.into_lua(lua)?;
        // Creates back-map from id to map index.
        self.to_index = self
            .map
            .objects
            .iter()
            .enumerate()
            .map(|(index, object)| (object.id(), index))
            .collect();

        // Add initializing moves(respond to signal).
        for (object, groups) in self.map.object_and_groups() {
            for group in groups.iter() {
                if let Some(id) = scripts.feature_names.lock().unwrap().get(group).copied() {
                    let features = scripts.features.lock().unwrap();
                    let feature = features
                        .get(&id)
                        .expect("Feature name should correspond to a feature implementation");
                    if let Some(movements) = feature.on_input(&layout, &signal, object.coord)? {
                        self.move_queue.extend(movements.into_iter());
                    }
                    if let Some(listens) = feature.get_listen(&layout, object.coord)? {
                        for target in listens.into_iter() {
                            self.listen.entry(target).or_default().push(object.id());
                        }
                    }
                }
            }
        }

        // Add subsequent moves generated by listeners.
        let mut index = 0;
        while index < self.move_queue.len() {
            let mut new_movements = Vec::<Movement>::new();
            let movement = &self.move_queue[index];
            let movement_value = movement.clone().into_lua(lua)?;
            if let Some(listeners) = self.listen.get(&movement.object) {
                for listener in listeners.iter() {
                    if let Some(listener) = self.to_index.get(listener) {
                        self.perform_listen(
                            *listener,
                            &layout,
                            &movement_value,
                            lua,
                            scripts,
                            &mut new_movements,
                        )?;
                    } else {
                        log::warn!("Non-existent listener provided by th script: {listener}");
                    }
                }
            }
            self.move_queue.extend(new_movements.into_iter());
            index += 1;
        }

        // Test if there is any objects that move two different ways,
        // and remove duplicate moves.
        self.move_queue.sort();
        let mut new_move_queue = Vec::<Movement>::new();
        if !self.move_queue.is_empty() {
            for current in self.move_queue.drain(..) {
                if let Some(previous) = new_move_queue.first()
                    && current.object == previous.object
                {
                    if current != *previous {
                        return Err(InfrError::DifferentMovements(
                            current.clone(),
                            previous.clone(),
                        ));
                    }
                } else {
                    new_move_queue.push(current);
                }
            }
        }
        self.move_queue = new_move_queue;

        // Sort the moves by their dependency relations.
        let graph = {
            let mut graph = crate::graph::Graph::<u32>::new();
            let mut map_object_node = HashMap::<u32, usize>::new();
            for movement in self.move_queue.iter() {
                let object = *map_object_node
                    .entry(movement.object)
                    .or_insert_with(|| graph.add(movement.object));
                let subsequent = *map_object_node
                    .entry(movement.required_by)
                    .or_insert_with(|| graph.add(movement.required_by));
                graph.connect(object, subsequent);
            }
            graph.scc()
        };

        let mut map_object_movement = self
            .move_queue
            .drain(..)
            .map(|movement| (movement.object, movement))
            .collect::<HashMap<_, _>>();

        // Perform the movements with dependency restrictions by topo sort.
        let mut performed_movements = Vec::new();
        let mut queue = VecDeque::new();
        let mut deg = vec![0; graph.len()];
        for u in 0..graph.len() {
            deg.push(graph.get_deg(u));
            if graph.get_deg(u) == 0 {
                queue.push_back(u);
            }
        }
        while let Some(u) = queue.pop_front() {
            let mut ok = true;
            for object in graph.get(u).iter() {
                let movement = map_object_movement
                    .get(object)
                    .expect("Movement should be in the map after inserting");
                if movement.forbid {
                    ok = false;
                    break;
                }
            }
            if ok {
                for object in graph.get(u).iter() {
                    let movement = map_object_movement
                        .remove(object)
                        .expect("Movement should be in the map and not removed");
                    performed_movements.push(movement.clone());
                    let object_id = self.perform_movement(movement)?;
                    // Synchronizes object ids for Add movement.
                    performed_movements.last_mut().unwrap().object = object_id;
                }
                for v in graph.get_next(u).iter().copied() {
                    deg[v] -= 1;
                    if deg[v] == 0 {
                        queue.push_back(v);
                    }
                }
            }
        }

        // Use two pointers to remove objects required.
        // This first performs the unique action.
        self.remove_queue = self
            .remove_queue
            .drain(..)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let mut new_objects = Vec::<Object>::new();
        {
            let mut iter = self.map.objects.drain(..).enumerate().fuse();
            for index in self.remove_queue.drain(..) {
                loop {
                    if let Some((current_index, object)) = iter.next() {
                        if current_index == index {
                            break;
                        } else {
                            new_objects.push(object);
                        }
                    }
                }
            }
            while let Some((_, object)) = iter.next() {
                new_objects.push(object);
            }
        }
        self.map.objects = new_objects;

        self.map.revert();
        Ok(performed_movements)
    }
}
