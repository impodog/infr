#![cfg(test)]

use infr_solver::*;

#[test]
fn test_layout1() {
    let mut map = Map::new();
    map.objects = vec![
        Object::new(Coord::new(0, 0), ObjectKind::Instance, "A".into()),
        Object::new(Coord::new(1, 1), ObjectKind::Structure, "A".into()),
        Object::new(
            Coord::new(1, 2),
            ObjectKind::Operator,
            consts::OPERATOR_ARROW.into(),
        )
        .with_direction(Direction::Up),
        Object::new(Coord::new(1, 3), ObjectKind::Structure, "B".into()),
    ];
    map.parse();
    map.build_solver();
    dbg!(&map.rules);
    map.push_object_state(0);
    assert!(map.prove_variable("A"));
    assert!(map.prove_variable("B"));
    map.revert();
    map.revert();
}

#[test]
fn test_layout_inconsistent() {
    let mut map = Map::new();
    map.objects = vec![
        Object::new(Coord::new(0, 0), ObjectKind::Instance, "A".into()),
        Object::new(Coord::new(1, 1), ObjectKind::Structure, "A".into()),
        Object::new(
            Coord::new(1, 2),
            ObjectKind::Operator,
            consts::OPERATOR_ARROW.into(),
        ),
        Object::new(Coord::new(1, 3), ObjectKind::Structure, "B".into()),
        Object::new(Coord::new(2, 1), ObjectKind::Structure, "B".into()),
        Object::new(
            Coord::new(2, 2),
            ObjectKind::Operator,
            consts::OPERATOR_ARROW.into(),
        ),
        Object::new(
            Coord::new(2, 3),
            ObjectKind::Operator,
            consts::OPERATOR_NOT.into(),
        ),
        Object::new(Coord::new(2, 4), ObjectKind::Structure, "A".into()),
    ];
    map.parse();
    map.build_solver();
    map.assert_variable("A");
    assert!(map.check_contradiction().is_err());
    map.revert();
}

#[test]
fn test_layout_overlap() {
    let mut map = Map::new();
    map.objects = vec![
        Object::new(Coord::new(0, 0), ObjectKind::Instance, "A".into()),
        Object::new(Coord::new(0, 0), ObjectKind::Instance, "B".into()),
        Object::new(Coord::new(3, 1), ObjectKind::Structure, "A".into()),
        Object::new(
            Coord::new(3, 2),
            ObjectKind::Operator,
            consts::OPERATOR_ARROW.into(),
        )
        .with_direction(Direction::Up),
        Object::new(
            Coord::new(3, 3),
            ObjectKind::Structure,
            consts::VAR_SOLID.into(),
        ),
        Object::new(Coord::new(1, 3), ObjectKind::Structure, "B".into()),
        Object::new(
            Coord::new(2, 3),
            ObjectKind::Operator,
            consts::OPERATOR_ARROW.into(),
        )
        .with_direction(Direction::Right),
    ];
    map.parse();
    map.build_solver();
    let relevant_groups = map.get_relevant_groups();
    map.prove_groups(relevant_groups);
    assert_eq!(map.check_overlap(), Err(Coord::new(0, 0)));
    map.revert();
}
