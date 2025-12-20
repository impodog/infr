use crate::consts;
use crate::prelude::*;
use std::sync::LazyLock;

/// 2d coordinates for infr objects.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}
impl Coord {
    /// Creates a coordinate.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl std::ops::Add for Coord {
    type Output = Coord;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl std::ops::Neg for Coord {
    type Output = Coord;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Undefined,
    Right,
    Up,
    Left,
    Down,
}
impl Direction {
    /// Returns the delta per move in that direction.
    pub fn delta(&self) -> Coord {
        match self {
            Self::Undefined => Coord::new(0, 0),
            Self::Right => Coord::new(1, 0),
            Self::Up => Coord::new(0, 1),
            Self::Left => Coord::new(-1, 0),
            Self::Down => Coord::new(0, -1),
        }
    }
}

/// The kind of the object, depending on how they make sentences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    /// Does not make a sentence.
    Instance,
    /// Only used to make a sentence.
    Structure,
    /// Both Instance and Structure.
    Symbol,
    /// Connects the sentences.
    Operator,
}
impl ObjectKind {
    /// Returns if the object kind makes sentences.
    pub fn makes_sentence(&self) -> bool {
        matches!(self, Self::Structure | Self::Symbol | Self::Operator)
    }
}

/// Stores all information required for solving logic.
#[derive(Debug, Clone)]
pub struct Object {
    pub id: i32,
    pub coord: Coord,
    pub kind: ObjectKind,
    /// The name of the object, indicating the set they belong to(Instance) / represent(Structure).
    /// All names should be CamelCase.
    pub name: String,
    /// The direction the object is facing.
    pub dir: Direction,
}
impl Object {
    /// If this object is an operator and has a valid name, a value representing the priority of
    /// the operator is returned.
    ///
    /// Operators with higher values are parsed first.
    pub fn priority(&self) -> Option<u8> {
        if self.kind == ObjectKind::Operator {
            match self.name.as_str() {
                consts::OP_SUBSET => Some(100),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Different types of rules(not axioms) that are constructed in the game by words.
#[derive(Debug, Clone)]
enum Rule {
    /// The former is a subset of the latter.
    Subset(String, String),
}

/// Stores a parsed node in the parser stack.
enum RuleOrSet {
    Rule(Rule),
    Set(String),
}
/// A stack storing relevant objects of a sentence.
#[derive(Default)]
struct RuleParser {
    stack: Vec<RuleOrSet>,
    ops: Vec<(String, u8)>,
    /// The direction the rules should be facing.
    dir: Direction,
    prev_coord: Coord,
}
impl RuleParser {
    /// Clears the parser for reusing allocated space.
    ///
    /// If there is only one rule in the stack, that rule is returned.
    fn clear(&mut self) -> Option<Rule> {
        self.ops.clear();
        if self.stack.len() == 1 {
            let node = self.stack.drain(..).next().unwrap();
            match node {
                RuleOrSet::Rule(rule) => Some(rule),
                _ => None,
            }
        } else {
            self.stack.clear();
            None
        }
    }

    /// Depending on operator type, creates a new rule node, or calls [`Self::clear`] on syntax
    /// error.
    ///
    /// The argument op must be valid (i.e. has a priority).
    fn merge_op_node(&mut self, op: &str) -> Option<Rule> {
        match op {
            consts::OP_SUBSET => {
                if self.stack.len() < 2 {
                    self.clear()
                } else {
                    let second = self.stack.pop().unwrap();
                    let first = self.stack.pop().unwrap();
                    if let (RuleOrSet::Set(first), RuleOrSet::Set(second)) = (first, second) {
                        self.stack
                            .push(RuleOrSet::Rule(Rule::Subset(first, second)));
                        None
                    } else {
                        self.clear()
                    }
                }
            }
            _ => {
                panic!("Operator {} is invalid", op);
            }
        }
    }

    /// Adds an object to the parsing sequence.
    ///
    /// If this object ends the previous valid sentence, the rule is returned.
    fn push(&mut self, object: Object) -> Option<Rule> {
        // If there is a gap between objects, the parser is cleared and the result is stored.
        //
        // A single object does not cause errors, therefore this result must be returned.
        let pending_result = if object.coord != self.prev_coord + self.dir.delta() {
            self.clear()
        } else {
            None
        };
        match object.kind {
            ObjectKind::Instance => pending_result.or(self.clear()),
            ObjectKind::Operator => {
                if let Some(priority) = object.priority()
                    && object.dir == self.dir
                {
                    while self
                        .ops
                        .last()
                        .is_some_and(|(_, prev_priority)| *prev_priority > priority)
                    {
                        let prev_op = self.ops.pop().unwrap();
                        if let Some(rule) = self.merge_op_node(prev_op.0.as_str()) {
                            // Failed rule.
                            return Some(rule);
                        }
                    }
                    self.ops.push((object.name, priority));
                    pending_result
                } else {
                    pending_result.or(self.clear())
                }
            }
            ObjectKind::Symbol | ObjectKind::Structure => {
                self.stack.push(RuleOrSet::Set(object.name));
                pending_result
            }
        }
    }

    /// Pops all remaining operators and return the last stored rule.
    ///
    /// This is a stronger version of [`Self::clear`].
    fn pop_all(&mut self) -> Option<Rule> {
        while let Some((op, _)) = self.ops.pop() {
            if let Some(rule) = self.merge_op_node(op.as_str()) {
                return Some(rule);
            }
        }
        self.clear()
    }
}

pub static FUNC_GETX: LazyLock<z3::Synchronized<z3::FuncDecl>> = LazyLock::new(|| {
    z3::FuncDecl::new(consts::FUNC_GETX, &[&z3::Sort::int()], &z3::Sort::int()).synchronized()
});
pub static FUNC_GETY: LazyLock<z3::Synchronized<z3::FuncDecl>> = LazyLock::new(|| {
    z3::FuncDecl::new(consts::FUNC_GETY, &[&z3::Sort::int()], &z3::Sort::int()).synchronized()
});
pub static FUNC_GET_TILE: LazyLock<z3::Synchronized<z3::FuncDecl>> = LazyLock::new(|| {
    z3::FuncDecl::new(
        consts::FUNC_GET_TILE,
        &[&z3::Sort::int(), &z3::Sort::int()],
        &z3::Sort::set(&z3::Sort::int()),
    )
    .synchronized()
});

/// Stores all values that represents game status.
#[derive(Debug, Clone)]
pub struct Game {
    timeout: u64,
    objects: Vec<Object>,
    /// Initially, asserts all the axioms possibly used in this game layout.
    solver: Solver,
}

impl Game {
    /// Initializes an empty game state.
    pub fn new(timeout: u64) -> Game {
        Self {
            timeout,
            objects: Default::default(),
            solver: Solver::new(),
        }
    }

    /// Adds an object to the list of all objects.
    pub fn add_object(&mut self, obj: Object) {
        self.objects.push(obj);
    }
    /// Returns an immutable reference to the array of objects.
    pub fn objects(&self) -> &[Object] {
        self.objects.as_slice()
    }

    /// Asserts an axiom.
    pub fn add_axiom(&self, axiom: z3::ast::Bool) {
        self.solver.assert(axiom);
    }

    /// Parses and adds all the game rules.
    fn add_rules(&self) {
        let mut objects = self.objects.clone();
        let mut parser = RuleParser::default();
        let mut rules = Vec::new();

        let update_rules =
            |objects: &Vec<Object>, parser: &mut RuleParser, rules: &mut Vec<Rule>| {
                for object in objects.iter() {
                    if let Some(rule) = parser.push(object.clone()) {
                        rules.push(rule);
                    }
                }
                if let Some(rule) = parser.pop_all() {
                    rules.push(rule);
                }
            };

        // Deal with vertical sentences from down to up.
        parser.dir = Direction::Up;
        objects.sort_unstable_by_key(|object| (object.coord.x, object.coord.y));
        update_rules(&objects, &mut parser, &mut rules);

        // From up to down.
        parser.clear();
        parser.dir = Direction::Down;
        objects.sort_unstable_by_key(|object| (object.coord.x, -object.coord.y));
        update_rules(&objects, &mut parser, &mut rules);

        // From left to right.
        parser.clear();
        parser.dir = Direction::Right;
        objects.sort_unstable_by_key(|object| (object.coord.y, object.coord.x));
        update_rules(&objects, &mut parser, &mut rules);

        // From right to left.
        parser.clear();
        parser.dir = Direction::Right;
        objects.sort_unstable_by_key(|object| (object.coord.y, object.coord.x));
        update_rules(&objects, &mut parser, &mut rules);

        use z3::ast::*;

        for rule in rules.into_iter() {
            match rule {
                Rule::Subset(subset, superset) => {
                    self.solver.assert(
                        Set::new_const(subset, &z3::Sort::int())
                            .set_subset(Set::new_const(superset, &z3::Sort::int())),
                    );
                }
            }
        }
        // Temporarily stored values of the same tile.
        let mut tile_list = Vec::new();
        let mut prev_coord = Coord::new(i32::MAX, i32::MAX);
        let update_solver = |tile_list: &Vec<Object>, prev_coord: Coord| {
            let set = Set::empty(&z3::Sort::int());
            for object in tile_list.iter() {
                set.add(&Int::from_i64(object.id as i64));
            }
            self.solver.assert(
                FUNC_GET_TILE
                    .recover()
                    .apply(&[
                        &Int::from_i64(prev_coord.x as i64),
                        &Int::from_i64(prev_coord.y as i64),
                    ])
                    .eq(&set),
            );
        };
        for object in objects.into_iter() {
            self.solver.assert(
                FUNC_GETX
                    .recover()
                    .apply(&[&Int::from_i64(object.id as i64)])
                    .eq(Int::from_i64(object.coord.x as i64)),
            );
            self.solver.assert(
                FUNC_GETY
                    .recover()
                    .apply(&[&Int::from_i64(object.id as i64)])
                    .eq(Int::from_i64(object.coord.y as i64)),
            );
            if object.coord != prev_coord && prev_coord.x != i32::MAX {
                update_solver(&tile_list, prev_coord);
                tile_list.clear();
                prev_coord = object.coord;
            }
            tile_list.push(object)
        }
        update_solver(&tile_list, prev_coord);
    }

    /// Initializes a Z3 config based on the fields of this game.
    fn get_config(&self) -> Config {
        let mut config = Config::new();
        config.set_model_generation(false);
        config.set_timeout_msec(self.timeout);
        config
    }

    /// Updates the solver to add the current state of the game, including object positions and
    /// in-game rules. These updates must be reverted by calling [`Self::revert`].
    pub fn update(&self) {
        self.solver.push();
        self.add_rules();
    }

    /// Reverts the solver from [`Self::update`] to remove the state of the game.
    pub fn revert(&self) {
        self.solver.pop(1);
    }
}
