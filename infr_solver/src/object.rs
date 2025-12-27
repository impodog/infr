//! Infr solver object representation.

use std::sync::atomic::{AtomicU32, Ordering};

/// The coordinates of the object in signed integer.
/// This allows infinite sized map.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Coord {
    /// Creates a new coordinates.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Creates an infinitely far coordinates.
    pub const fn infinite() -> Self {
        Self::new(i32::MAX, i32::MAX)
    }

    /// Adds two coordinates, wrapping around arithmetic overflow.
    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self {
            x: self.x.wrapping_add(rhs.x),
            y: self.y.wrapping_add(rhs.y),
        }
    }
}
impl std::ops::Add for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Neg for Coord {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl std::ops::Sub for Coord {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Represents directions of all purposes in ccw order.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    #[default]
    Right,
    Up,
    Left,
    Down,
}
impl Direction {
    /// An array of all directions in ccw order.
    pub const DIRECTIONS: [Direction; 4] = [
        Direction::Right,
        Direction::Up,
        Direction::Left,
        Direction::Down,
    ];
    /// Returns the delta vector of the direction.
    pub const fn delta(self) -> Coord {
        match self {
            Direction::Right => Coord::new(1, 0),
            Direction::Up => Coord::new(0, 1),
            Direction::Left => Coord::new(-1, 0),
            Direction::Down => Coord::new(0, -1),
        }
    }
}

/// The kind of the object, depending the position they take in sentences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// It never makes a sentence.
    Instance,
    /// It can only be used in a sentence.
    Structure,
    /// Both `Instance` and `Structure`.
    Symbol,
    /// This is not a group, but an operator that connects groups.
    Operator,
}

/// Represents base information of a single object.
/// Additional properties may be bound with object id.
#[derive(Debug, Clone)]
pub struct Object {
    /// Globally unique identifier of the object.
    id: u32,
    /// Coordinates relative to (0, 0).
    pub coord: Coord,
    /// Direction of the object. If `None`, the object's direction is undetermined.
    pub direction: Option<Direction>,
    pub kind: ObjectKind,
    /// For Instance/Structure/Symbol, this is the name of the group it belongs to or represents.
    /// For Operator, this is the type of the operator.
    pub group: String,
}

/// Atomic counter for generating unique object IDs.
static GLOBAL_ID: AtomicU32 = AtomicU32::new(1);

impl Object {
    /// Creates a new object with a fresh id.
    pub fn new(coord: Coord, kind: ObjectKind, group: String) -> Self {
        Self {
            id: GLOBAL_ID.fetch_add(1, Ordering::SeqCst),
            coord,
            direction: None,
            kind,
            group,
        }
    }

    /// Returns the unique id of the object.
    pub fn id(&self) -> u32 {
        self.id
    }
}

/// Represents a text of a logical `Rule`.
#[derive(Debug, Clone)]
pub enum Text {
    IsGroup(String),
}
/// Represents a `Text` with or without a negation.
#[derive(Debug, Clone)]
pub struct SignedText(pub Text, pub bool);

/// Represents a logic that can be asserted.
#[derive(Debug, Clone)]
pub struct Rule {
    pub premise: Vec<SignedText>,
    pub conclusion: SignedText,
}

#[derive(Debug, Clone)]
pub struct Map {
    pub objects: Vec<Object>,
    solver: z3::Solver,
}

impl Map {
    /// Creates an empty map.
    pub fn new() -> Self {
        let solver = z3::Solver::new();
        Self {
            objects: Default::default(),
            solver,
        }
    }

    /// Adds an object to the map.
    pub fn push(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Asserts an axiom before parsing map-local rules.
    pub fn assert(&self, expr: &z3::ast::Bool) {
        self.solver.assert(expr);
    }

    /// Takes a largest bite from the objects within range start..end, returning the number of object used.
    /// This should be called repeatedly to ensure all sentences are parsed.
    fn parse_sentence(&mut self, start: usize, end: usize) -> usize {
        todo!()
    }

    /// Parses map-local rules, which can be reverted later.
    pub fn parse(&mut self) {
        for direction in Direction::DIRECTIONS {
            let delta = direction.delta();
            // Sort objects by their coordinates relative to the direction.
            self.objects.sort_by_key(|object| {
                if delta.x == 0 {
                    (object.coord.x, object.coord.y * delta.y)
                } else {
                    (object.coord.y, object.coord.x * delta.x)
                }
            });
            let mut start: usize = 0;
            let mut prev_coord = Coord::infinite();
            for index in 0..self.objects.len() {
                // This only parses sentences if they are consecutive.
                if prev_coord.wrapping_add(delta) != self.objects[index].coord {
                    self.parse_sentence(start, index);
                    start = index;
                }
                prev_coord = self.objects[index].coord;
            }
        }
    }
}
