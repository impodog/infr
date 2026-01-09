use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use mlua::prelude::*;

use infr_macros::Id;

/// Stores the sha256 key of a path, for hot reloading.
#[derive(Debug, Clone)]
struct ScriptContent {
    path: PathBuf,
    key: String,
}

type ArcMap<K, V> = Arc<Mutex<HashMap<K, V>>>;

#[derive(Default, Debug)]
/// Stores the imported scripts and manages hot reloading.
pub struct Scripts {
    scripts: HashMap<String, ScriptContent>,
    pub features: ArcMap<FeatureId, Feature>,
    /// Stores the map from feature name to its id.
    pub feature_names: ArcMap<String, FeatureId>,
    pub instances: ArcMap<InstanceId, Instance>,
}

impl Scripts {
    /// Creates a new scripts manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the module name and path to the script manager.
    /// This module won't be loaded until `Self::reload` is called.
    pub fn add<'a, 'b>(&mut self, name: impl Into<Cow<'a, str>>, path: impl Into<Cow<'b, Path>>) {
        let name = name.into();
        let path = path.into();
        if !self.scripts.contains_key(name.as_ref()) {
            self.scripts.insert(
                name.into_owned(),
                ScriptContent {
                    path: path.into_owned(),
                    key: String::new(),
                },
            );
        }
    }

    /// Adds Rust API to the lua environment.
    pub fn add_global_functions(&self, lua: &Lua) -> LuaResult<()> {
        let globals = lua.globals();

        globals.set("UNUSED_ID", infr_solver::consts::UNUSED_ID)?;

        {
            let features = self.features.clone();
            let features_names = self.feature_names.clone();
            globals.set(
                "register_feature",
                lua.create_function(move |_, feature: Feature| -> LuaResult<FeatureId> {
                    let id = FeatureId::new();
                    features_names
                        .lock()
                        .unwrap()
                        .insert(feature.name.clone(), id);
                    features.lock().unwrap().insert(id, feature);
                    Ok(id)
                })?,
            )?;
        }

        {
            let instances = self.instances.clone();
            globals.set(
                "register_instance",
                lua.create_function(move |_, instance: Instance| -> LuaResult<InstanceId> {
                    let id = InstanceId::new();
                    instances.lock().unwrap().insert(id, instance);
                    Ok(id)
                })?,
            )?;
        }

        Ok(())
    }

    /// Reloads all scripts, if they are changed or never loaded.
    pub fn reload(&mut self, lua: &Lua) -> LuaResult<()> {
        for (name, path) in self.scripts.iter_mut() {
            let content = std::fs::read(path.path.as_path());
            match content {
                Ok(content) => {
                    // Check if the content has changed.
                    let key = sha256::digest(&content);
                    if key != path.key {
                        log::info!("Reloading module contents {}", name);
                        lua.unload_module(name)?;
                        lua.preload_module(
                            name,
                            lua.create_function(move |lua: &Lua, _name: String| {
                                lua.load(content.clone()).exec()
                            })?,
                        )?;
                        path.key = key;
                    }
                }
                Err(err) => {
                    log::error!("Unable to load script {:?} : {}", path.path, err);
                }
            }
        }
        Ok(())
    }
}

/// Defined by the script, describes how objects with that feature will respond to different events.
#[derive(FromLua, Debug, Clone)]
pub struct Feature {
    pub name: String,
    /// Function to execute when inputted with a signal.
    pub on_input: Option<LuaFunction>,
    /// Function that returns all objects it should listen to.
    pub get_listen: Option<LuaFunction>,
    /// Function to execute when the listened object performs a movement.
    pub respond: Option<LuaFunction>,
}
#[derive(Debug, Id)]
pub struct FeatureId(u32);

impl Feature {
    /// Runs internal lua function if it is present, and return the array of movements returned by the script.
    pub fn on_input(
        &self,
        layout: &LuaValue,
        signal: &LuaValue,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<crate::Movement>>> {
        if let Some(on_input) = self.on_input.as_ref() {
            let result = on_input.call::<Vec<crate::Movement>>((signal, layout, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Runs internal lua function if it is present, and return the array of objects it should listen to.
    pub fn get_listen(
        &self,
        layout: &LuaValue,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<u32>>> {
        if let Some(get_listen) = self.get_listen.as_ref() {
            let result = get_listen.call::<Vec<u32>>((layout, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub fn respond(
        &self,
        layout: &LuaValue,
        movement: &LuaValue,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<crate::Movement>>> {
        if let Some(respond) = self.respond.as_ref() {
            let result = respond.call::<Vec<crate::Movement>>((layout, movement, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

/// Defined by the script, creates an instance class with sprite info.
#[derive(FromLua, Debug, Clone)]
pub struct Instance {
    /// The feature tied to all instances.
    pub feature: FeatureId,
}
#[derive(Debug, Id)]
pub struct InstanceId(u32);

// -- Implement relevant lua conversions.
impl IntoLua for crate::Manner {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        match self {
            Self::Swipe(direction) => {
                table.set("type", "Swipe")?;
                table.set("direction", direction)?;
            }
            Self::Remove => {
                table.set("type", "Remove")?;
            }
            Self::Teleport => {
                table.set("type", "Teleport")?;
            }
            Self::Add(object_desc) => {
                table.set("type", "Add")?;
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
        table.set("required_by", self.required_by)?;
        table.set("forbid", self.forbid)?;
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
                let required_by = table.get::<u32>("required_by")?;
                let forbid = table.get::<bool>("forbid")?;
                Ok(Self {
                    manner,
                    object,
                    dest,
                    required_by,
                    forbid,
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
