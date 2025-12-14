use crate::prelude::*;

/// 2d coordinates for infr objects.
pub struct Coord {
    pub x: u32,
    pub y: u32,
}
/// The kind of the object, depending on whether they make sentences.
pub enum ObjectKind {
    /// Does not make a sentence.
    Instance,
    /// Only used to make a sentence.
    Structure,
    /// Both Instance and Structure.
    Symbol,
}
/// Stores all information required for solving logic.
pub struct Object {
    pub id: i32,
    pub coord: Coord,
    pub kind: ObjectKind,
    /// The name of the object, indicating the set they belong to(Instance) / represent(Structure).
    pub name: String,
}

pub struct Game {
    config: Config,
    objects: Vec<Object>,
}

impl Game {
    /// Initializes an empty game state.
    pub fn new(timeout: u64) -> Game {
        let mut config = Config::new();
        config.set_timeout_msec(timeout);
        Self {
            config,
            objects: Default::default(),
        }
    }

    /// Adds an object to the object vector.
    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(obj);
    }
}

