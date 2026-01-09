use infr_solver::*;

impl From<Direction> for crate::Direction {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Right => crate::Direction(1),
            Direction::Up => crate::Direction(2),
            Direction::Left => crate::Direction(3),
            Direction::Down => crate::Direction(4),
        }
    }
}
impl From<Option<Direction>> for crate::Direction {
    fn from(value: Option<Direction>) -> Self {
        match value {
            Some(Direction::Right) => crate::Direction(1),
            Some(Direction::Up) => crate::Direction(2),
            Some(Direction::Left) => crate::Direction(3),
            Some(Direction::Down) => crate::Direction(4),
            None => crate::Direction(0),
        }
    }
}
impl From<crate::Direction> for Option<Direction> {
    fn from(value: crate::Direction) -> Self {
        match value.0 {
            1 => Some(Direction::Right),
            2 => Some(Direction::Up),
            3 => Some(Direction::Left),
            4 => Some(Direction::Down),
            _ => None,
        }
    }
}

impl From<Coord> for crate::Coord {
    fn from(value: Coord) -> Self {
        crate::Coord(value.x, value.y)
    }
}
impl From<crate::Coord> for Coord {
    fn from(value: crate::Coord) -> Self {
        Coord::new(value.0, value.1)
    }
}

impl crate::Map {
    /// Constructs a map for transferring by cloning relevant information.
    ///
    /// This must be done after calling `Map::prove_groups`, see it for other dependencies.
    pub fn from_map(map: &Map) -> Self {
        let mut result = Vec::new();
        for (index, object) in map.objects.iter().enumerate() {
            result.push(crate::Object {
                id: object.id(),
                coord: object.coord.into(),
                direction: object.direction.into(),
                groups: map
                    .groups()
                    .get(index)
                    .expect("\"from_map\" should be called after proving groups")
                    .clone(),
                nature: match &object.kind {
                    ObjectKind::Instance => format!("{}-I", object.group),
                    ObjectKind::Structure => format!("{}-S", object.group),
                    _ => object.group.clone(),
                },
            });
        }
        Self(result)
    }
}
