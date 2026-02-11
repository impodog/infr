//! Implements necessary lua conversions.

use crate::*;
use mlua::prelude::*;

impl IntoLua for Direction {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Self::Right => lua.create_string("Right"),
            Self::Up => lua.create_string("Up"),
            Self::Left => lua.create_string("Left"),
            Self::Down => lua.create_string("Down"),
        }
        .map(LuaValue::String)
    }
}
impl FromLua for Direction {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(s) => {
                let s = s.to_str();
                match s.as_ref().map(LuaBorrowedStr::as_ref) {
                    Ok("Right") => Ok(Self::Right),
                    Ok("Up") => Ok(Self::Up),
                    Ok("Left") => Ok(Self::Left),
                    Ok("Down") => Ok(Self::Down),
                    _ => Err(LuaError::FromLuaConversionError {
                        from: "string",
                        to: "Direction".to_owned(),
                        message: Some("Invalid direction string".to_string()),
                    }),
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Direction".to_owned(),
                message: Some("Invalid direction value".to_string()),
            }),
        }
    }
}

impl IntoLua for Coord {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("x", self.x)?;
        table.set("y", self.y)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for Coord {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let x = table.get::<i32>("x")?;
                let y = table.get::<i32>("y")?;
                Ok(Self { x, y })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Coord".to_owned(),
                message: Some("Invalid coord value".to_string()),
            }),
        }
    }
}

impl IntoLua for ObjectKind {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Self::Instance => lua.create_string("Instance"),
            Self::Symbol => lua.create_string("Symbol"),
            Self::Structure => lua.create_string("Structure"),
            Self::Operator => lua.create_string("Operator"),
        }
        .map(LuaValue::String)
    }
}
impl FromLua for ObjectKind {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(s) => {
                let s = s.to_str();
                match s.as_ref().map(LuaBorrowedStr::as_ref) {
                    Ok("Instance") => Ok(Self::Instance),
                    Ok("Symbol") => Ok(Self::Symbol),
                    Ok("Structure") => Ok(Self::Structure),
                    Ok("Operator") => Ok(Self::Operator),
                    _ => Err(LuaError::FromLuaConversionError {
                        from: "string",
                        to: "ObjectKind".to_owned(),
                        message: Some("Invalid object kind string".to_string()),
                    }),
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ObjectKind".to_owned(),
                message: Some("Invalid object kind value".to_string()),
            }),
        }
    }
}

/// A minimal object description produced by (or given to) scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDesc {
    /// This is unused when the scripts sends a new object.
    pub id: u32,
    pub direction: Option<Direction>,
    pub kind: ObjectKind,
    pub group: Vec<String>,
    pub flags: Vec<String>,
}
impl IntoLua for ObjectDesc {
    fn into_lua(self, lua: &Lua) -> Result<LuaValue, LuaError> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("direction", self.direction)?;
        table.set("kind", self.kind)?;
        table.set("group", self.group)?;
        table.set("flags", self.flags)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for ObjectDesc {
    fn from_lua(value: LuaValue, _lua: &Lua) -> Result<Self, LuaError> {
        match value {
            LuaValue::Table(table) => {
                if table.get::<Option<u32>>("id")?.is_some() {
                    log::warn!("Object id is unused in script-sent object description.");
                }
                let direction = table.get("direction")?;
                let kind = table.get("kind")?;
                let group = table.get("group")?;
                let flags = table.get("flags").unwrap_or_default();
                Ok(ObjectDesc {
                    id: crate::consts::UNUSED_ID,
                    direction,
                    kind,
                    group,
                    flags,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ObjectDesc".to_owned(),
                message: Some("Invalid object description value".to_string()),
            }),
        }
    }
}

impl Map {
    /// Gets the object description at object index.
    /// Returns `None` if the index does not exist, or if the object groups are not parsed yet.
    pub fn get_object_desc(&self, index: usize) -> Option<ObjectDesc> {
        let object = self.objects.get(index)?;
        let group = self.groups().get(index)?;
        Some(ObjectDesc {
            id: object.id(),
            direction: object.direction,
            kind: object.kind,
            group: group.clone(),
            flags: object.flags().clone(),
        })
    }
}

impl IntoLua for &SignedText {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        match &self.0 {
            Text::IsGroup(group) => {
                table.set("kind", "IsGroup")?;
                table.set("group", group.clone())?;
            }
        }
        table.set("neg", self.1)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for SignedText {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match &value {
            LuaValue::String(group) => {
                let group = group.to_string_lossy();
                if let Some(group) = group.strip_prefix('!') {
                    Ok(Self(Text::IsGroup(group.to_owned()), true))
                } else {
                    Ok(Self(Text::IsGroup(group), false))
                }
            }
            LuaValue::Table(table) => {
                let kind = table.get::<String>("kind")?;
                let text = match kind.as_str() {
                    "IsGroup" => {
                        let group = table.get::<String>("group")?;
                        Text::IsGroup(group)
                    }
                    _ => {
                        return Err(LuaError::FromLuaConversionError {
                            from: value.type_name(),
                            to: "SignedText".to_owned(),
                            message: Some(format!("Unexpected text kind: {kind}")),
                        });
                    }
                };
                let neg = table.get::<bool>("neg").unwrap_or_default();
                Ok(Self(text, neg))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "SignedText".to_owned(),
                message: Some("Invalid signed text value".to_string()),
            }),
        }
    }
}

impl IntoLua for &Rule {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        let premise = lua.create_table()?;
        for (index, premise_text) in self.premise.iter().enumerate() {
            premise.set(index + 1, premise_text)?;
        }
        table.set("premise", premise)?;
        let conclusion = lua.create_table()?;
        for (index, conclusion_text) in self.conclusion.iter().enumerate() {
            conclusion.set(index + 1, conclusion_text)?;
        }
        table.set("conclusion", conclusion)?;
        table.set("double_arrow", self.double_arrow)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for Rule {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let premise = table.get::<Vec<SignedText>>("premise")?;
                let conclusion = table.get::<Vec<SignedText>>("conclusion")?;
                let double_arrow = table.get::<bool>("double_arrow")?;
                Ok(Self {
                    premise,
                    conclusion,
                    double_arrow,
                    range: Default::default(),
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Rule".to_owned(),
                message: Some("Invalid rule value".to_string()),
            }),
        }
    }
}
