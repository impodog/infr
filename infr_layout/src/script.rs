use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use mlua::prelude::*;

/// Stores the sha256 key of a path, for hot reloading.
struct ScriptContent {
    path: PathBuf,
    key: String,
}

#[derive(Default)]
/// Stores the imported scripts and manages hot reloading.
pub struct Scripts {
    scripts: HashMap<String, ScriptContent>,
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
