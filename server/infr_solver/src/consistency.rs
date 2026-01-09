//! Implements functions related to checking the logical consistency of the map.

use crate::{Coord, Map, consts};

/// The return result of `Map::check_contradiction`, describing rules and axioms that causes a contradiction.
#[derive(Default, Debug, Clone)]
pub struct Contradiction {
    pub rules: Vec<usize>,
    pub axioms: Vec<usize>,
}

impl Map {
    /// Checks if the map is logically inconsistent, returning any errors.
    pub fn check_contradiction(&self) -> Result<(), Contradiction> {
        let sat = self.solver.check();
        if sat == z3::SatResult::Unsat {
            let mut contradiction = Contradiction::default();
            for unsat in self.solver.get_unsat_core().into_iter() {
                let name = unsat.to_string();
                if name.starts_with(consts::RULE_PREFIX) {
                    let index = &name[consts::RULE_PREFIX.len()..];
                    if let Ok(index) = index.parse::<usize>() {
                        contradiction.rules.push(index);
                    }
                }
                if name.starts_with(consts::AXIOM_PREFIX) {
                    let index = &name[consts::AXIOM_PREFIX.len()..];
                    if let Ok(index) = index.parse::<usize>() {
                        contradiction.axioms.push(index);
                    }
                }
            }
            Err(contradiction)
        } else {
            Ok(())
        }
    }

    /// Returns if any two solid objects overlap each other, creating a violation.
    /// Returns the coordinates if a overlap is detected.
    ///
    /// You should call `Self::prove_groups` before this function.
    pub fn check_overlap(&self) -> Result<(), Coord> {
        // Objects should be sorted so far.
        let mut prev_coord = Coord::infinite();
        let mut has_solid = Option::<bool>::None;
        for index in 0..self.objects.len() {
            if prev_coord != self.objects[index].coord {
                prev_coord = self.objects[index].coord;
                has_solid = None;
            }
            let is_solid = self.groups()[index]
                .binary_search_by(|s| s.as_str().cmp(consts::VAR_SOLID))
                .is_ok();
            if is_solid {
                if has_solid.is_some_and(|x| x) {
                    return Err(prev_coord);
                } else {
                    has_solid = Some(true);
                }
            }
        }
        Ok(())
    }
}
