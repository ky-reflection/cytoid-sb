use crate::host::table_to_authoring_map;
use cytoid_sb_builder::{ControllerHandle, StoryboardBuilder};
use mlua::{UserData, UserDataMethods};
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct LuaController {
    handle: ControllerHandle,
    builder: Arc<Mutex<StoryboardBuilder>>,
}

impl LuaController {
    pub fn new(handle: ControllerHandle, builder: Arc<Mutex<StoryboardBuilder>>) -> Self {
        Self { handle, builder }
    }
}

impl UserData for LuaController {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("key", |lua, this, (time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = table_to_authoring_map(lua, json_patch)?;
            this.builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_controller_keyframe(this.handle, time, Value::Object(patch))
                .map_err(mlua::Error::runtime)?;
            Ok(())
        });
        methods.add_method("rel", |lua, this, (add_time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = table_to_authoring_map(lua, json_patch)?;
            this.builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_controller_relative_keyframe(this.handle, add_time, Value::Object(patch))
                .map_err(mlua::Error::runtime)?;
            Ok(())
        });
    }
}
