use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
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

    pub level_callbacks: ArcMap<LevelCallbackId, LevelCallback>,
    pub level_callback_names: ArcMap<String, LevelCallbackId>,

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
                    let id = *features_names
                        .lock()
                        .unwrap()
                        .entry(feature.name.clone())
                        .or_insert_with(FeatureId::new);

                    log::info!("Registered feature {} with id {id:?}", feature.name);

                    features.lock().unwrap().insert(id, feature);
                    Ok(id)
                })?,
            )?;
        }

        {
            let level_callbacks = self.level_callbacks.clone();
            let level_callback_names = self.level_callback_names.clone();
            module.set(
                "register_level_callback",
                lua.create_function(
                    move |_, callback: LevelCallback| -> LuaResult<LevelCallbackId> {
                        let id = *level_callback_names
                            .lock()
                            .unwrap()
                            .entry(callback.name.clone())
                            .or_insert_with(LevelCallbackId::new);

                        log::info!("Registered level callback {} with id {id:?}", callback.name);

                        level_callbacks.lock().unwrap().insert(id, callback);

                        Ok(id)
                    },
                )?,
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

    /// Executes all level callbacks whose flag requirements meet the given ones.
    pub fn run_level_callbacks(&self, _lua: &Lua, flags: &HashSet<String>) -> LuaResult<()> {
        let level_callbacks = self.level_callbacks.lock().unwrap();
        for callback in level_callbacks.values() {
            let mut ok = true;
            for flag in callback.flags.iter().map(String::as_str) {
                #[allow(clippy::manual_strip)]
                let (reverse, base) = if flag.starts_with('!') {
                    (true, &flag[1..])
                } else {
                    (false, flag)
                };
                let search_for = format!("S:Call:{base}");
                // If not reversed, flags must contain it; If reverse, flags must not contain it.
                if !(reverse ^ flags.contains(&search_for)) {
                    ok = false;
                    break;
                }
            }
            if ok {
                callback.callback.call::<()>(())?;
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

/// Returned by the script, either listens to a object or a coord.
#[derive(Debug, Clone, Copy)]
pub enum ListenKind {
    Object(u32),
    Coord(infr_solver::Coord),
}

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
    ) -> LuaResult<Option<Vec<ListenKind>>> {
        if let Some(get_listen) = self.get_listen.as_ref() {
            let result = get_listen.call::<Vec<ListenKind>>((layout, object, coord))?;
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

/// Defined by the script, called every time a level loads.
#[derive(Debug, Clone)]
pub struct LevelCallback {
    pub name: String,
    /// It requires these 'S:Call:' flags to exist(without '!' symbol) / not exist(with '!' symbol) to run.
    pub flags: Vec<String>,
    pub callback: LuaFunction,
}

#[derive(Debug, Id)]
pub struct LevelCallbackId(u32);
