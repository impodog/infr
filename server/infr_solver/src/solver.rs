//! Infr solver object representation.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::consts::{self, ABSTRACT_NATURE};

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
    /// Additional flags for special cases.
    flags: Vec<String>,
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
            flags: Default::default(),
        }
    }

    /// Sets the direction of the object.
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Sets the flags of the object.
    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.set_flags(flags);
        self
    }
    /// Replaces the flags of the object with a given vector.
    pub fn set_flags(&mut self, flags: Vec<String>) {
        self.flags = flags;
        self.flags.sort();
    }
    /// Returns the reference to the object's flags.
    pub fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    /// Searches for all flags that starts with this prefix.
    /// This will automatically strip the prefix.
    ///
    /// Note: The flags must be sorted(auto done by the object).
    pub fn search_flags_with<'f>(
        flags: &'f [String],
        pat: &str,
    ) -> impl std::iter::Iterator<Item = &'f str> {
        fn take_slice(s: &str, len: usize) -> &str {
            if len < s.len() { &s[..len] } else { s }
        }

        let lower_bound = {
            let mut l = 0;
            let mut r = flags.len();
            while l < r {
                let mid = (l + r) >> 1;
                if flags[mid].as_str() < pat {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            l
        };
        let upper_bound = {
            let mut l = 0;
            let mut r = flags.len();
            while l < r {
                let mid = (l + r) >> 1;
                if take_slice(flags[mid].as_str(), pat.len()) <= pat {
                    l = mid + 1;
                } else {
                    r = mid;
                }
            }
            l
        };
        if lower_bound < upper_bound {
            flags[lower_bound..upper_bound].iter()
        } else {
            flags[0..0].iter()
        }
        .map(|s| &s[pat.len()..])
    }

    /// Convenience method for searching flags of this object that start with `pat`.
    pub fn search_flags<'f>(&'f self, pat: &str) -> impl std::iter::Iterator<Item = &'f str> {
        Self::search_flags_with(self.flags().as_slice(), pat)
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
    pub(crate) range: (Coord, Coord),
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
    /// Generated by `Self::prove_groups`, storing the relevant groups of each object.
    /// The groups are guaranteed to be sorted.
    groups: Vec<Vec<String>>,
    /// These groups are always added into relevant groups.
    always_check: BTreeSet<String>,
    pub(crate) solver: z3::Solver,
}

impl Map {
    /// Creates an empty map.
    pub fn new() -> Self {
        let solver = z3::Solver::new();
        Self {
            objects: Default::default(),
            rules: Default::default(),
            groups: Default::default(),
            always_check: Default::default(),
            solver,
        }
    }

    /// Returns a vector storing the groups each object belongs to.
    /// In the second dimension all strings are guaranteed to be sorted.
    pub fn groups(&self) -> &Vec<Vec<String>> {
        &self.groups
    }

    /// Returns a zipped iterator over the objects and their groups.
    pub fn object_and_groups(&self) -> impl Iterator<Item = (&Object, &Vec<String>)> {
        self.objects.iter().zip(self.groups.iter())
    }

    /// Adds an object to the map.
    pub fn push(&mut self, object: Object) {
        self.objects.push(object);
    }

    /// Asserts a z3 expression for general purposes.
    pub fn assert(&self, expr: &z3::ast::Bool) {
        self.solver.assert(expr);
    }

    /// Asserts a variable, used for debug purposes.
    pub fn assert_variable(&self, value: impl Into<String>) {
        self.solver.assert(z3::ast::Bool::new_const(value.into()));
    }

    /// Asserts an axiom before parsing map-local rules.
    /// This axiom is then tracked and the id is returned if any inconsistency was found.
    pub fn assert_axiom(&self, id: u32, expr: &z3::ast::Bool) {
        self.solver.assert_and_track(
            expr,
            &z3::ast::Bool::new_const(format!("{}{}", consts::AXIOM_PREFIX, id)),
        );
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
            self.parse_all_sentences(start, self.objects.len());
        }
    }

    /// Build the solver according to rules parsed.
    /// You must first call `Self::parse` before this, and call `Self::revert` after using the solver.
    ///
    /// After this point, the objects' order should not be changed until the next player action.
    /// This allows you to build `Self::groups`.
    pub fn build_solver(&mut self) {
        use z3::ast;

        /// Folds the list of texts into an assertable bool.
        fn fold_condition(conditions: &[SignedText]) -> ast::Bool {
            conditions
                .iter()
                .fold(Option::<ast::Bool>::None, |prev, text| {
                    let cond = match text.0 {
                        Text::IsGroup(ref group) => ast::Bool::new_const(group.to_string()),
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
            let rule_name = format!("{}{}", consts::RULE_PREFIX, index);
            // First update the function map to include all possibly used names.
            let premise = fold_condition(rule.premise.as_slice());
            let conclusion = fold_condition(rule.conclusion.as_slice());
            let statement = if rule.double_arrow {
                premise.eq(conclusion)
            } else {
                premise.implies(conclusion)
            };
            self.solver
                .assert_and_track(statement, &ast::Bool::new_const(rule_name));
        }
    }

    /// Reverts rules added by `Self::build_solver` or `Self::push_object_state`.
    pub fn revert(&self) {
        self.solver.pop(1);
    }

    /// Asserts all relevant states to the object at a certain index.
    ///
    /// You must revert the solver using `Self::revert` after checking the object state.
    ///
    /// Panics if index is out of bounds.
    pub fn push_object_state(&self, index: usize) {
        use z3::ast;

        self.solver.push();
        let object = self.objects.get(index).expect("index should be in bounds.");
        if matches!(object.kind, ObjectKind::Instance | ObjectKind::Symbol) {
            self.solver
                .assert(ast::Bool::new_const(object.group.clone()));
        } else {
            self.solver
                .assert(ast::Bool::new_const(ABSTRACT_NATURE.to_owned()));
        }
    }

    /// For each Instance/Symbol and each relevant group given, proves and stores if it is in that group.
    ///
    /// The objects' order should not changed after calling `Self::build_solver`, so you can correspond each stored `Self::groups` with each object.
    pub fn prove_groups(&mut self, relevant_groups: &BTreeSet<String>) {
        let mut groups = Vec::<Vec<String>>::new();
        for (index, object) in self.objects.iter().enumerate() {
            let mut current_groups = BTreeSet::new();
            self.push_object_state(index);
            for group in relevant_groups.iter() {
                if self.prove_variable(group) {
                    current_groups.insert(group.to_string());
                }
            }
            self.revert();
            // Special case: The object's natural group is not in the relevant groups,
            // but we still need to add it.
            if matches!(object.kind, ObjectKind::Instance | ObjectKind::Symbol)
                && !relevant_groups.contains(&object.group)
            {
                current_groups.insert(object.group.clone());
            }
            for flag in object.search_flags("S:Always:") {
                current_groups.insert(flag.to_owned());
            }
            groups.push(current_groups.into_iter().collect());
        }
        self.groups = groups;
    }

    /// This group is always returned in relevant groups.
    pub fn always_check<'s>(&mut self, group: impl Into<String>) {
        self.always_check.get_or_insert(group.into());
    }

    /// Walks all rules and return relevant groups that can be used for `Self::prove_group`.
    pub fn get_relevant_groups(&self) -> BTreeSet<String> {
        let mut groups = BTreeSet::new();
        for rule in self.rules.iter() {
            for text in rule.premise.iter().chain(rule.conclusion.iter()) {
                text.walk_func_names(|name| {
                    groups.insert(name.to_string());
                    Result::<(), ()>::Ok(())
                })
                .expect("The function should always return Ok");
            }
        }
        let mut always_check = self.always_check.clone();
        groups.append(&mut always_check);
        groups
    }

    /// Returns if the condition can be proven in the current state.
    /// You need to call `Self::push_object_state`
    ///
    /// The condition is considered false if it cannot be proven within timeout(in milliseconds).
    ///
    /// To set the timeout, you should call [`z3::with_z3_config`] and wrap any calls within that closure.
    pub fn prove(&self, value: &z3::ast::Bool) -> bool {
        // z3::with_z3_config takes too long to build.
        let result = self.solver.check_assumptions(&[value.not()]);
        result == z3::SatResult::Unsat
    }

    /// Returns if the variable can be proven in the current state.
    /// See `Self::prove` for details.
    pub fn prove_variable(&self, variable: impl Into<String>) -> bool {
        self.prove(&z3::ast::Bool::new_const(variable.into()))
    }
}
