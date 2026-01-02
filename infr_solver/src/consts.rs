//! Infr solver base constants.
//! This file should not be imported with use::*, as the constants may have duplicate names and mess up with matching.
//!
//! All identifier constants follow CamelCase.

pub const OPERATOR_NOT: &str = "Not";
pub const OPERATOR_AND: &str = "And";
pub const OPERATOR_OR: &str = "Or";
pub const OPERATOR_ARROW: &str = "Arrow";
pub const OPERATOR_DOUBLE_ARROW: &str = "DoubleArrow";

/// The prefix for tracking map-specific rule inconsistency.
pub const RULE_PREFIX: &str = "Rule";
/// The prefix for tracking axiom inconsistency with rules.
pub const AXIOM_PREFIX: &str = "Axiom";

/// The variable in the solver, testing whether the object is solid (i.e. cannot overlap other solid objects).
pub const VAR_SOLID: &str = "Solid";

/// Placeholder for scripts sending a new object without an id.
pub const UNUSED_ID: u32 = 0xfeedd095;
