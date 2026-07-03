use crate::host::table_to_authoring_map;
use cytoid_sb_builder::{SpriteHandle, StoryboardBuilder};
use mlua::{UserData, UserDataMethods};
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LuaSprite {
    handle: SpriteHandle,
    builder: Arc<Mutex<StoryboardBuilder>>,
}

impl LuaSprite {
    pub fn new(handle: SpriteHandle, builder: Arc<Mutex<StoryboardBuilder>>) -> Self {
        Self { handle, builder }
    }
}

impl UserData for LuaSprite {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("key", |lua, this, (time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = table_to_authoring_map(lua, json_patch)?;
            this.builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_sprite_keyframe(this.handle, time, Value::Object(patch))
                .map_err(mlua::Error::runtime)?;
            Ok(())
        });
        methods.add_method("rel", |lua, this, (add_time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = table_to_authoring_map(lua, json_patch)?;
            this.builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_sprite_relative_keyframe(this.handle, add_time, Value::Object(patch))
                .map_err(mlua::Error::runtime)?;
            Ok(())
        });
    }
}
