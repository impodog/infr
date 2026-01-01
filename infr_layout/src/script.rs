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
    lua: Lua,
    scripts: HashMap<String, ScriptContent>,
    pub sprites: ArcMap<SpriteId, Sprite>,
    pub sprite_sheets: ArcMap<SpriteSheetId, SpriteSheet>,
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
        if self.scripts.contains_key(name.as_ref()) {
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

        {
            let sprites = self.sprites.clone();
            globals.set(
                "register_sprite",
                lua.create_function(move |_, sprite: Sprite| -> LuaResult<SpriteId> {
                    let id = SpriteId::new();
                    sprites.lock().unwrap().insert(id, sprite);
                    Ok(id)
                })?,
            )?;
        }

        {
            let sprite_sheets = self.sprite_sheets.clone();
            globals.set(
                "register_sprite_sheet",
                lua.create_function(move |_, sheet: SpriteSheet| -> LuaResult<SpriteSheetId> {
                    let id = SpriteSheetId::new();
                    sprite_sheets.lock().unwrap().insert(id, sheet);
                    Ok(id)
                })?,
            )?;
        }

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
                        lua.preload_module(
                            name,
                            lua.create_function(move |lua: &Lua, name: String| {
                                log::info!("Reloading module {}", name);
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

/// Provided by the script, stores information on how a sprite should be displayed.
#[derive(FromLua, Clone, Debug)]
pub struct Sprite {
    /// The path to the image file.
    pub sheet: PathBuf,
    /// Size of each cell on the image.
    pub size: (u32, u32),
    /// Number of cells in the grid.
    pub grid: (u32, u32),
    /// Interval between frames in milliseconds.
    pub interval: u32,
}
#[derive(Debug, Id)]
pub struct SpriteId(u32);

/// Defines the sprites of an object, with optional directional sprites.
#[derive(FromLua, Debug, Clone)]
pub struct SpriteSheet {
    pub idle: Sprite,
    pub direction: Option<[Sprite; 4]>,
}
#[derive(Debug, Id)]
pub struct SpriteSheetId(u32);

/// Defined by the script, describes how objects with that feature will respond to different events.
#[derive(FromLua, Debug, Clone)]
pub struct Feature {
    pub name: String,
    /// The sprite sheet for the word of this feature.
    pub sprite_word: SpriteSheetId,
    /// Function to execute when inputted with a direction.
    pub on_input: Option<LuaFunction>,
}
#[derive(Debug, Id)]
pub struct FeatureId(u32);

/// Defined by the script, creates an instance class with sprite info.
#[derive(FromLua, Debug, Clone)]
pub struct Instance {
    /// The feature tied to all instances.
    pub feature: FeatureId,
    /// The sprite for instances of this feature.
    pub sprite_instance: SpriteSheetId,
}
#[derive(Debug, Id)]
pub struct InstanceId(u32);
