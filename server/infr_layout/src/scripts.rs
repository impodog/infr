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
    /// Scripts may apply for a step-specific table that correlates with an object.
    pub grabbed_tables: ArcMap<u32, LuaValue>,
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
        let module = lua.create_table()?;

        module.set("UNUSED_ID", infr_solver::consts::UNUSED_ID)?;
        module.set("DIRECTIONS", ["Right", "Up", "Left", "Down"])?;

        {
            let features = self.features.clone();
            let features_names = self.feature_names.clone();
            module.set(
                "register_feature",
                lua.create_function(move |_, feature: Feature| -> LuaResult<FeatureId> {
                    let id = FeatureId::new();
                    features_names
                        .lock()
                        .unwrap()
                        .insert(feature.name.clone(), id);

                    log::info!("Registered feature {} with id {id:?}", feature.name);

                    features.lock().unwrap().insert(id, feature);
                    Ok(id)
                })?,
            )?;
        }

        {
            let instances = self.instances.clone();
            module.set(
                "register_instance",
                lua.create_function(move |_, instance: Instance| -> LuaResult<InstanceId> {
                    let id = InstanceId::new();
                    instances.lock().unwrap().insert(id, instance);
                    Ok(id)
                })?,
            )?;
        }

        {
            let grabbed_tables = self.grabbed_tables.clone();
            module.set(
                "grab_table",
                lua.create_function(move |lua: &Lua, object: u32| -> LuaResult<LuaValue> {
                    let mut grabbed_tables = grabbed_tables.lock().unwrap();
                    let value = match grabbed_tables.entry(object) {
                        std::collections::hash_map::Entry::Occupied(occupied) => {
                            occupied.get().clone()
                        }
                        std::collections::hash_map::Entry::Vacant(vacant) => {
                            let table = lua.create_table()?;
                            vacant.insert(LuaValue::Table(table)).clone()
                        }
                    };
                    Ok(value)
                })?,
            )?;
        }

        {
            let table = lua.create_table()?;
            table.set(
                "move",
                lua.create_function(
                    |_,
                     (coord, direction): (infr_solver::Coord, infr_solver::Direction)|
                     -> LuaResult<infr_solver::Coord> {
                        Ok(coord + direction.delta())
                    },
                )?,
            )?;
            module.set("Coord", table)?;
        }

        lua.globals().set("infr", module)?;

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
#[derive(Debug, Clone)]
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
        object: u32,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<crate::Movement>>> {
        if let Some(on_input) = self.on_input.as_ref() {
            let result =
                on_input.call::<Vec<crate::Movement>>((layout.clone(), signal, object, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Runs internal lua function if it is present, and return the array of objects it should listen to.
    pub fn get_listen(
        &self,
        layout: &LuaValue,
        object: u32,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<u32>>> {
        if let Some(get_listen) = self.get_listen.as_ref() {
            let result = get_listen.call::<Vec<u32>>((layout, object, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub fn respond(
        &self,
        layout: &LuaValue,
        movement: &LuaValue,
        object: u32,
        coord: infr_solver::Coord,
    ) -> LuaResult<Option<Vec<crate::Movement>>> {
        if let Some(respond) = self.respond.as_ref() {
            let result = respond.call::<Vec<crate::Movement>>((layout, movement, object, coord))?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

/// Defined by the script, creates an instance class with sprite info.
#[derive(Debug, Clone)]
pub struct Instance {
    /// The feature tied to all instances.
    pub feature: FeatureId,
}
#[derive(Debug, Id)]
pub struct InstanceId(u32);
