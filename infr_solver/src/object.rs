//! Infr solver object representation.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::consts;

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

    /// Returns `Self::kind` and reference to `Self::group` as a tuple,
    /// used for pattern matching.
    pub fn kind_and_group(&self) -> (ObjectKind, &str) {
        (self.kind, self.group.as_str())
    }
}

/// Represents a text of a logical `Rule`.
#[derive(Debug, Clone)]
pub enum Text {
    IsGroup(String),
}
/// Represents a `Text` with or without a negation.
///
/// The second field is true when there is a negation.
#[derive(Debug, Clone)]
pub struct SignedText(pub Text, pub bool);

impl SignedText {
    pub fn negate(self) -> Self {
        Self(self.0, !self.1)
    }

    /// Walks through the function names in the text using the provided closure.
    ///
    /// The closure may return an error, and the first error is returned.
    pub fn walk_func_names<F, E>(&self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&str) -> Result<(), E>,
    {
        match self.0 {
            Text::IsGroup(ref name) => f(name),
        }
    }
}

impl From<Text> for SignedText {
    fn from(value: Text) -> Self {
        Self(value, false)
    }
}

/// Represents a logic that can be asserted.
#[derive(Debug, Clone)]
pub struct Rule {
    range: (Coord, Coord),
    // NOTE: Currently there is only support for AND connections.
    pub premise: Vec<SignedText>,
    pub conclusion: Vec<SignedText>,
    /// `true` when the rule is connected by a double arrow, single arrow otherwise.
    pub double_arrow: bool,
}

impl Rule {
    /// Returns the range of objects where the rule is parsed.
    /// This is always of width 1, or height 1.
    pub fn range(&self) -> (Coord, Coord) {
        self.range
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub objects: Vec<Object>,
    pub rules: Vec<Rule>,
    solver: z3::Solver,
}

impl Map {
    /// Creates an empty map.
    pub fn new() -> Self {
        let solver = z3::Solver::new();
        Self {
            objects: Default::default(),
            rules: Default::default(),
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

    /// Takes a largest bite of single text from the objects within range [start, end),
    /// returning the text and number of objects used.
    ///
    /// If this position does not form a text, (None, 0) is returned.
    ///
    /// Note that you must ensure [start, end) in within length of objects.
    fn parse_text(&mut self, start: usize, end: usize) -> (Option<SignedText>, usize) {
        if end == start {
            return (None, 0);
        }
        match self.objects[start].kind_and_group() {
            (ObjectKind::Structure | ObjectKind::Symbol, group) => {
                (Some(Text::IsGroup(group.to_string()).into()), 1)
            }
            (ObjectKind::Operator, consts::OPERATOR_NOT) => {
                let text = self.parse_text(start + 1, end);
                (text.0.map(SignedText::negate), text.1)
            }
            _ => (None, 0),
        }
    }

    /// Takes a largest bite from the objects within range [start, end), returning the number of objects viewed.
    /// This should be called repeatedly to ensure all sentences are parsed.
    ///
    /// This will consume objects while syntax errors occur. So even if no new sentences are added,
    ///
    /// Note that you must ensure [start, end) in within length of objects.
    fn parse_sentence(&mut self, start: usize, end: usize) -> usize {
        let mut current_index = start;
        let mut after_arrow = false;
        let mut double_arrow = false;
        let mut premise = Vec::new();
        let mut conclusion = Vec::new();
        if let (Some(text), offset) = self.parse_text(start, end) {
            premise.push(text);
            current_index += offset;
            while current_index < end {
                match self.objects[current_index].kind_and_group() {
                    (ObjectKind::Operator, consts::OPERATOR_AND) => {
                        if let (Some(text), offset) = self.parse_text(current_index + 1, end) {
                            if after_arrow {
                                conclusion.push(text);
                            } else {
                                premise.push(text);
                            }
                            current_index += offset + 1;
                        } else {
                            break;
                        }
                    }
                    (ObjectKind::Operator, op)
                        if matches!(op, consts::OPERATOR_ARROW | consts::OPERATOR_DOUBLE_ARROW)
                            && !after_arrow =>
                    {
                        after_arrow = true;
                        double_arrow = op == consts::OPERATOR_DOUBLE_ARROW;
                        if let (Some(text), offset) = self.parse_text(current_index + 1, end) {
                            conclusion.push(text);
                            current_index += offset + 1;
                        } else {
                            break;
                        }
                    }
                    _ => {
                        // Error occured on current_index, and the rule is ready for the final test.
                        break;
                    }
                }
            }
            // Test if the rule is grammatical.
            if !premise.is_empty() && !conclusion.is_empty() {
                self.rules.push(Rule {
                    range: (
                        self.objects[start].coord,
                        self.objects[current_index - 1].coord,
                    ),
                    premise,
                    conclusion,
                    double_arrow,
                });
            }
            current_index - start
        } else {
            // This is a syntax error as premise can't be empty, so we consume the object and return 1.
            1
        }
    }

    /// Parses all sentences in the given range by calling `Self::parse_sentence` repeatedly.
    fn parse_all_sentences(&mut self, start: usize, end: usize) {
        let mut current_index = start;
        while current_index < end {
            let sentence_length = self.parse_sentence(current_index, end);
            debug_assert!(sentence_length != 0);
            current_index += sentence_length;
        }
    }

    /// Parses map-local rules and stores them in `Self::rules`.
    ///
    /// This will clear any rules present before parsing.
    pub fn parse(&mut self) {
        self.rules.clear();

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
                    self.parse_all_sentences(start, index);
                    start = index;
                }
                // If an operator faces the wrong direction, they are skipped.
                if self.objects[index].kind == ObjectKind::Operator
                    && self.objects[index]
                        .direction
                        .is_some_and(|object_direction| object_direction != direction)
                {
                    self.parse_all_sentences(start, index);
                    start = index;
                    prev_coord = Coord::infinite();
                } else {
                    prev_coord = self.objects[index].coord;
                }
            }
        }
    }

    /// Build the solver according to rules parsed and object status.
    /// You must first call `Self::parse` before this, and call `Self::revert` after using the solver.
    pub fn build_solver(&mut self) {
        use std::collections::HashMap;
        use z3::ast;

        let mut functions = HashMap::<String, z3::FuncDecl>::new();

        /// Initializes a function entry in the map, if not already present.
        fn add_func_entry<'functions>(
            functions: &'functions mut HashMap<String, z3::FuncDecl>,
            func_name: &str,
        ) -> &'functions z3::FuncDecl {
            functions.entry(func_name.to_string()).or_insert_with(|| {
                z3::FuncDecl::new(func_name, &[&z3::Sort::int()], &z3::Sort::bool())
            })
        }

        /// Get a function declaration from the map, unwrapping the result.
        fn get_func_unwrap<'functions>(
            functions: &'functions HashMap<String, z3::FuncDecl>,
            func_name: &str,
        ) -> &'functions z3::FuncDecl {
            functions
                .get(func_name)
                .expect("This function should be added by walking the text")
        }

        /// Folds the list of texts into a assertable bool.
        fn fold_condition(
            functions: &HashMap<String, z3::FuncDecl>,
            conditions: &[SignedText],
            variable: &ast::Int,
        ) -> ast::Bool {
            conditions
                .iter()
                .fold(Option::<ast::Bool>::None, |prev, text| {
                    let cond = match text.0 {
                        Text::IsGroup(ref group) => {
                            let func = get_func_unwrap(functions, group);
                            func.apply(&[variable])
                                .as_bool()
                                .expect("Function result should be bool")
                        }
                    };
                    let cond = if text.1 { cond.not() } else { cond };
                    if let Some(prev) = prev {
                        Some(prev & cond)
                    } else {
                        Some(cond)
                    }
                })
                .expect("Conditions should not be empty.")
        }

        self.solver.push();
        for (index, rule) in self.rules.iter().enumerate() {
            // Create a unique rule name for tracking logical errors.
            let rule_name = format!("Rule{}", index);
            let variable_name = format!("Var{}", index);
            // We use ints equal to the object's id.
            let variable = ast::Int::new_const(variable_name);
            // First update the function map to include all possibly used names.
            for text in rule.premise.iter().chain(rule.conclusion.iter()) {
                text.walk_func_names(|func_name| -> Result<(), ()> {
                    add_func_entry(&mut functions, func_name);
                    Ok(())
                })
                .expect("The provided closure should always return Ok.");
            }
            self.solver.assert_and_track(
                fold_condition(&functions, rule.premise.as_slice(), &variable),
                &ast::Bool::new_const(rule_name),
            );
        }

        // Add object-specific states.
        for object in self.objects.iter() {
            let func = add_func_entry(&mut functions, object.group.as_str());
            self.solver.assert(
                func.apply(&[&z3::ast::Int::from_i64(object.id as i64)])
                    .as_bool()
                    .expect("This function should return bool."),
            );
        }
    }

    /// Reverts rules added by `Self::build_solver`.
    pub fn revert(&self) {
        self.solver.pop(1);
    }

    /// Returns if the condition can be proven in the current state.
    ///
    /// The condition is considered false if it cannot be proven within timeout(in milliseconds).
    pub fn prove(&self, timeout: u64, value: &z3::ast::Bool) -> bool {
        use z3::PrepareSynchronized;

        let mut config = z3::Config::new();
        config.set_timeout_msec(timeout);
        let solver = self.solver.synchronized();
        let assumption = value.not().synchronized();
        z3::with_z3_config(&config, || {
            let result = solver.recover().check_assumptions(&[assumption.recover()]);
            result == z3::SatResult::Unsat
        })
    }

    /// Proves that a object is in this group. For details see `Self::prove`.
    pub fn prove_in_group(&self, timeout: u64, id: i32, group: &str) -> bool {
        self.prove(
            timeout,
            &z3::FuncDecl::new(group, &[&z3::Sort::int()], &z3::Sort::bool())
                .apply(&[&z3::ast::Int::from_i64(id as i64)])
                .as_bool()
                .expect("This function should return bool"),
        )
    }
}
