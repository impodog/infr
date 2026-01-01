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
}

/// Stores movement destination and the manner of movement.
#[derive(Debug, Clone)]
pub struct Movement {
    pub manner: Manner,
    /// The id of the object that sends the movement.
    pub sender: u32,
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

    /// Initializes the movements with a signal. This will also clear any previous record of movement or errors.
    ///
    /// You should call `Self::step` repeatedly to parse the subsequent movements.
    /// You must call `Self::clear` after all movements are parsed.
    pub fn init(&mut self, input: Signal) -> Result<(), InfrError> {
        todo!()
    }
}
