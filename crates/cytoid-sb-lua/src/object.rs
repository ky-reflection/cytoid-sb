use crate::host::table_to_authoring_map;
use cytoid_sb_builder::{LineHandle, StoryboardBuilder, TextHandle, VideoHandle};
use mlua::{UserData, UserDataMethods};
use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum TimelineObjectHandle {
    Text(TextHandle),
    Video(VideoHandle),
    Line(LineHandle),
}

#[derive(Clone)]
pub struct LuaTimelineObject {
    handle: TimelineObjectHandle,
    builder: Arc<Mutex<StoryboardBuilder>>,
}

impl LuaTimelineObject {
    pub fn new(handle: TimelineObjectHandle, builder: Arc<Mutex<StoryboardBuilder>>) -> Self {
        Self { handle, builder }
    }
}

impl UserData for LuaTimelineObject {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("key", |lua, this, (time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = Value::Object(table_to_authoring_map(lua, json_patch)?);
            let mut builder = this
                .builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?;
            match this.handle {
                TimelineObjectHandle::Text(handle) => {
                    builder.push_text_keyframe(handle, time, patch)
                }
                TimelineObjectHandle::Video(handle) => {
                    builder.push_video_keyframe(handle, time, patch)
                }
                TimelineObjectHandle::Line(handle) => {
                    builder.push_line_keyframe(handle, time, patch)
                }
            }
            .map_err(mlua::Error::runtime)?;
            Ok(())
        });
        methods.add_method("rel", |lua, this, (add_time, patch): (f64, mlua::Table)| {
            let json_patch = mlua::Value::Table(patch);
            let patch = Value::Object(table_to_authoring_map(lua, json_patch)?);
            let mut builder = this
                .builder
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?;
            match this.handle {
                TimelineObjectHandle::Text(handle) => {
                    builder.push_text_relative_keyframe(handle, add_time, patch)
                }
                TimelineObjectHandle::Video(handle) => {
                    builder.push_video_relative_keyframe(handle, add_time, patch)
                }
                TimelineObjectHandle::Line(handle) => {
                    builder.push_line_relative_keyframe(handle, add_time, patch)
                }
            }
            .map_err(mlua::Error::runtime)?;
            Ok(())
        });
    }
}
