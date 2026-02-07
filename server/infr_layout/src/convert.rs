//! Implements conversions between Rust and Lua.

use mlua::prelude::*;

impl IntoLua for crate::Manner {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        match self {
            Self::Placeholder => {
                table.set("kind", "Placeholder")?;
            }
            Self::Swipe(direction) => {
                table.set("kind", "Swipe")?;
                table.set("direction", direction)?;
            }
            Self::Remove => {
                table.set("kind", "Remove")?;
            }
            Self::Teleport => {
                table.set("kind", "Teleport")?;
            }
            Self::Add(object_desc) => {
                table.set("kind", "Add")?;
                table.set("object", object_desc)?;
            }
        }
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for crate::Manner {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(kind) => {
                let kind = kind.to_str();
                match kind.as_ref().map(LuaBorrowedStr::as_ref) {
                    Ok("Remove") => Ok(Self::Remove),
                    Ok("Teleport") => Ok(Self::Teleport),
                    Ok("Placeholder") => Ok(Self::Placeholder),
                    _ => Err(LuaError::FromLuaConversionError {
                        from: "string",
                        to: "Manner".to_owned(),
                        message: Some(
                            "For manner shorthand forms, only Remove and Teleport are allowed"
                                .to_owned(),
                        ),
                    }),
                }
            }
            LuaValue::Table(table) => {
                let kind = table.get::<String>("kind")?;
                match kind.as_str() {
                    "Placeholder" => Ok(Self::Placeholder),
                    "Swipe" => {
                        let direction = table.get::<infr_solver::Direction>("direction")?;
                        Ok(Self::Swipe(direction))
                    }
                    "Remove" => Ok(Self::Remove),
                    "Teleport" => Ok(Self::Teleport),
                    "Add" => {
                        let object_desc = table.get::<infr_solver::ObjectDesc>("object")?;
                        Ok(Self::Add(object_desc))
                    }
                    _ => Err(LuaError::FromLuaConversionError {
                        from: "string",
                        to: "Manner".to_owned(),
                        message: Some(format!("Invalid kind {kind} for Manner")),
                    }),
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "Manner".to_owned(),
                message: Some("Invalid value type for Manner".to_owned()),
            }),
        }
    }
}

impl IntoLua for crate::Movement {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("manner", self.manner)?;
        table.set("object", self.object)?;
        table.set("dest", self.dest)?;
        table.set("prereqs", self.prereqs)?;
        table.set("postreqs", self.postreqs)?;
        table.set("disables", self.disables)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for crate::Movement {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let manner = table.get::<crate::Manner>("manner")?;
                let object = table.get::<u32>("object")?;
                let dest = table.get::<infr_solver::Coord>("dest")?;
                let prereqs = table.get::<Vec<u32>>("prereqs").unwrap_or_default();
                let postreqs = table.get::<Vec<u32>>("postreqs").unwrap_or_default();
                let disables = table.get::<Vec<u32>>("disables").unwrap_or_default();
                Ok(Self {
                    manner,
                    object,
                    dest,
                    prereqs,
                    postreqs,
                    disables,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "Movement".to_owned(),
                message: Some("Invalid value type for Movement".to_owned()),
            }),
        }
    }
}

impl IntoLua for crate::Signal {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("direction", self.direction)?;
        table.set("round", self.round)?;
        Ok(LuaValue::Table(table))
    }
}
impl FromLua for crate::Signal {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let direction = table.get("direction")?;
                let round = table.get("round")?;
                Ok(Self { direction, round })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "Signal".to_owned(),
                message: Some("Invalid value type for Signal".to_owned()),
            }),
        }
    }
}

// IntoLua is not implemented, as this can only be returned by the script.
impl FromLua for crate::scripts::ListenKind {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Integer(integer) => Ok(Self::Object(integer as u32)),
            LuaValue::Table(_) => {
                let coord = infr_solver::Coord::from_lua(value, lua)?;
                Ok(Self::Coord(coord))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "ListenKind".to_owned(),
                message: Some("Invalid value type for ListenKind".to_owned()),
            }),
        }
    }
}

impl FromLua for crate::scripts::Feature {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let name = table.get("name")?;
                let on_input = table.get("on_input")?;
                let get_listen = table.get("get_listen")?;
                let respond = table.get("respond")?;
                Ok(Self {
                    name,
                    on_input,
                    get_listen,
                    respond,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "Feature".to_owned(),
                message: Some("Invalid value type for Feature".to_owned()),
            }),
        }
    }
}

impl FromLua for crate::scripts::Instance {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Table(table) => {
                let feature = table.get("feature")?;
                Ok(Self { feature })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "value",
                to: "Instance".to_owned(),
                message: Some("Invalid value type for Instance".to_owned()),
            }),
        }
    }
}
