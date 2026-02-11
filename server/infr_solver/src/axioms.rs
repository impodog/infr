use std::{collections::HashMap, sync::LazyLock};

use crate::*;
use z3::ast::*;

/// An axiom function should register its containing z3 Bool into the map using the given id.
pub type AxiomFunc = fn(u32, &mut Map);

fn word_pushable_axiom(id: u32, map: &mut Map) {
    map.assert_axiom(
        id,
        &Bool::new_const(consts::ABSTRACT_NATURE).implies(Bool::new_const(consts::WORD_PUSH)),
    );
    map.always_check(consts::WORD_PUSH);
}

pub fn add_axioms(map: &mut Map, ids: impl IntoIterator<Item = u32>) {
    for id in ids {
        match id {
            1 => word_pushable_axiom(id, map),
            _ => {
                log::warn!("Unknown axiom id number {id}");
            }
        }
    }
}

macro_rules! slice {
    [$($args: expr),+] => {
        [$($args),+].as_slice()
    };
}
pub static AXIOM_GROUPS: LazyLock<HashMap<&'static str, &'static [u32]>> = LazyLock::new(|| {
    HashMap::<&'static str, &'static [u32]>::from_iter([
        ("WordPushable", slice![1]),
        ("All", slice![1]),
    ])
});
