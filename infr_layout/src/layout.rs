use crate::scripts::*;
use mlua::prelude::*;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};

use infr_solver::*;

/// Describes the manner of movement.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Movement {
    pub manner: Manner,
    /// The id of the object that executes the movement.
    pub object: u32,
    pub dest: Coord,
}

/// A signal from player input. Lua functions can respond to this accordingly.
pub struct Signal {
    pub direction: Option<Direction>,
}

/// Errors that occur when the layout is parsing movements.
pub enum InfrError {
    /// A logical contradiction defined by rules and axioms.
    Contradiction(infr_solver::Contradiction),
    /// Two solid objects in the same tile.
    Overlap(Coord),
    /// The external script returns an unexpected error.
    Script(mlua::Error),
}
impl From<infr_solver::Contradiction> for InfrError {
    fn from(value: infr_solver::Contradiction) -> Self {
        Self::Contradiction(value)
    }
}
impl From<Coord> for InfrError {
    fn from(value: Coord) -> Self {
        Self::Overlap(value)
    }
}
impl From<mlua::Error> for InfrError {
    fn from(value: mlua::Error) -> Self {
        Self::Script(value)
    }
}

pub type ArcMap = Arc<RwLock<Map>>;

/// Represents the game layout and parses movements.
#[derive(Debug)]
pub struct Layout {
    pub map: Map,
    move_queue: Vec<Movement>,
}

impl Layout {
    /// Creates an empty game layout. To add an object, use methods from `self.map`.
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            move_queue: Default::default(),
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
        Ok(relevant_groups)
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

    /// Initializes the movements with a signal. This will also clear any previous record of movement or errors.
    ///
    /// You should call `Self::step` repeatedly to parse the subsequent movements.
    /// You must call `Self::clear` after all movements are parsed.
    pub fn init(&mut self, input: Signal, lua: &Lua, scripts: &Scripts) -> Result<(), InfrError> {
        let relevant_groups = self.init_map()?;
        let table = self.convert_to_lua(lua)?;
        for (object, groups) in self.map.object_and_groups() {
            for group in groups.iter() {
                if let Some(id) = scripts.feature_names.lock().unwrap().get(group).copied() {
                    let features = scripts.features.lock().unwrap();
                    match features.get(&id) {
                        Some(feature) => {
                            if let Some(ref on_input) = feature.on_input {
                                todo!()
                            }
                        }
                        None => {
                            log::error!(
                                concat!(
                                    "Feature {} is registered without feature implementation. ",
                                    "This shouldn't be possible with lua scripts."
                                ),
                                group
                            );
                        }
                    }
                }
            }
        }
        self.map.revert();
        Ok(())
    }
}
