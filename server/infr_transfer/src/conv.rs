//! This module is only compiled on the server side.
//! It implements conversions between solver types and transfer types.
//!
//! All crate-names are invoked with a prefix, while solver types are default.

use infr_layout::*;
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

impl From<Manner> for crate::MoveManner {
    fn from(value: Manner) -> Self {
        match value {
            Manner::Placeholder => {
                panic!(
                    "Manner Placeholder should be filtered before converting to infr_transfer::MoveManner"
                )
            }
            Manner::Swipe(direction) => Self::Swipe(direction.into()),
            Manner::Add(_) => Self::Add,
            Manner::Remove => Self::Remove,
            Manner::Teleport => Self::Teleport,
        }
    }
}
impl From<Movement> for crate::Movement {
    fn from(value: Movement) -> Self {
        crate::Movement {
            object: value.object,
            manner: value.manner.into(),
            dest: value.dest.into(),
        }
    }
}
impl From<&Movement> for crate::Movement {
    fn from(value: &Movement) -> Self {
        crate::Movement {
            object: value.object,
            manner: value.manner.clone().into(),
            dest: value.dest.into(),
        }
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
                    ObjectKind::Instance => format!("${}", object.group),
                    ObjectKind::Structure => format!("@{}", object.group),
                    ObjectKind::Symbol => format!("%{}", object.group),
                    ObjectKind::Operator => format!("={}", object.group),
                },
                flags: object.flags().clone(),
            });
        }
        Self(result)
    }

    /// Gets objects in corresponding order to the id iterator.
    /// If an id is not present in the map, an `Err` value containing that id is returned.
    pub fn from_map_objects(
        map: &Map,
        objects: impl IntoIterator<Item = crate::ObjectId>,
    ) -> Result<Self, crate::ObjectId> {
        let mut result = Vec::new();
        let mut indices = Vec::new();
        for (index, object) in map.objects.iter().enumerate() {
            indices.push((object.id(), index));
        }
        indices.sort_by_key(|(id, _)| *id);
        for id in objects {
            let Ok(vector_index) = indices.binary_search_by_key(&id, |(id, _)| *id) else {
                return Err(id);
            };
            let index = indices[vector_index].1;
            let object = map.objects.get(index).unwrap();
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
                    ObjectKind::Instance => format!("${}", object.group),
                    ObjectKind::Structure => format!("@{}", object.group),
                    ObjectKind::Symbol => format!("%{}", object.group),
                    ObjectKind::Operator => format!("={}", object.group),
                },
                flags: object.flags().clone(),
            });
        }
        Ok(Self(result))
    }
}

impl From<String> for crate::ServerError {
    fn from(value: String) -> Self {
        crate::ServerError::ServerSide(value)
    }
}

/// Utility conversion, allowing using ? for quick error propagation.
impl From<crate::ServerError> for axum::response::ErrorResponse {
    fn from(value: crate::ServerError) -> Self {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        let status_code = match value {
            crate::ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            crate::ServerError::ServerSide(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::PARTIAL_CONTENT,
        };
        let response = (status_code, axum::Json(value)).into_response();
        axum::response::ErrorResponse::from(response)
    }
}

impl TryFrom<crate::LevelObject> for Object {
    type Error = crate::ServerError;

    fn try_from(value: crate::LevelObject) -> Result<Self, Self::Error> {
        let mut object = match value.group.chars().next() {
            Some('$') => Object::new(
                value.coord.into(),
                ObjectKind::Instance,
                value.group[1..].to_owned(),
            ),
            Some('@') => Object::new(
                value.coord.into(),
                ObjectKind::Structure,
                value.group[1..].to_owned(),
            ),
            Some('%') => Object::new(
                value.coord.into(),
                ObjectKind::Symbol,
                value.group[1..].to_owned(),
            ),
            Some('=') => Object::new(
                value.coord.into(),
                ObjectKind::Operator,
                value.group[1..].to_owned(),
            ),
            onset => {
                return Err(crate::ServerError::BadRequest(format!(
                    "Unidentified object onset: {onset:?}"
                )));
            }
        };
        object.direction = value.direction.into();
        object.set_flags(value.flags);
        Ok(object)
    }
}
