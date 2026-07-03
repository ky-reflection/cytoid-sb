use crate::controller::LuaController;
use crate::object::{LuaTimelineObject, TimelineObjectHandle};
use crate::sprite::LuaSprite;
use camino::Utf8Path;
use cytoid_sb_builder::{
    lua_table_to_authoring_map, lua_table_to_inline_controller, StoryboardBuilder,
};
use cytoid_sb_diag::{SbError, SbResult};
use cytoid_sb_model::StoryboardDocument;
use mlua::{Function, Lua, LuaOptions, LuaSerdeExt, StdLib, Table, UserData, Value};
use serde_json::{json, Map, Value as JsonValue};
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
struct NullMarker;

impl UserData for NullMarker {}

pub fn compile_file(path: &Utf8Path) -> SbResult<StoryboardDocument> {
    let source = fs::read_to_string(path.as_std_path()).map_err(|source| SbError::Io {
        path: path.to_string(),
        source,
    })?;
    compile_source(path.as_str(), &source)
}

pub fn compile_source(path: &str, source: &str) -> SbResult<StoryboardDocument> {
    let lua = Lua::new_with(
        StdLib::MATH | StdLib::STRING | StdLib::TABLE,
        LuaOptions::default(),
    )
    .map_err(|err| SbError::Other {
        message: format!("failed to initialize Lua VM: {err}"),
    })?;
    let env = configure_sandbox(&lua).map_err(|err| lua_error(path, err))?;
    let builder = Arc::new(Mutex::new(StoryboardBuilder::new()));
    register_sb_api(&lua, &env, builder.clone()).map_err(|err| lua_error(path, err))?;

    if let Err(err) = lua.load(source).set_name(path).set_environment(env).exec() {
        return Err(SbError::Other {
            message: format!("Lua error in {path}: {err}"),
        });
    }

    let doc = builder
        .lock()
        .map_err(|_| SbError::Other {
            message: "sb builder mutex poisoned".into(),
        })?
        .snapshot();

    Ok(doc)
}

fn lua_error(path: &str, err: mlua::Error) -> SbError {
    SbError::Other {
        message: format!("Lua error in {path}: {err}"),
    }
}

fn configure_sandbox(lua: &Lua) -> mlua::Result<Table> {
    let globals = lua.globals();
    let env = lua.create_table()?;

    for key in [
        "assert", "error", "ipairs", "next", "pairs", "pcall", "select", "tonumber", "tostring",
        "type",
    ] {
        env.set(key, globals.get::<Function>(key)?)?;
    }

    for key in ["math", "string", "table"] {
        env.set(key, globals.get::<Table>(key)?)?;
    }

    let mt = lua.create_table()?;
    mt.set(
        "__index",
        lua.create_function(|_, key: Value| -> mlua::Result<Value> {
            Err(mlua::Error::runtime(format!(
                "global '{}' is not available in sandbox",
                lua_key_name(key)?
            )))
        })?,
    )?;
    env.set_metatable(Some(mt))?;
    Ok(env)
}

fn lua_key_name(key: Value) -> mlua::Result<String> {
    match key {
        Value::String(s) => Ok(s.to_str()?.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        other => Ok(format!("{other:?}")),
    }
}

fn register_sb_api(
    lua: &Lua,
    env: &Table,
    builder: Arc<Mutex<StoryboardBuilder>>,
) -> mlua::Result<()> {
    let sb = lua.create_table()?;
    let builder_for_sprite = builder.clone();

    sb.set(
        "sprite",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            let handle = builder_for_sprite
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .create_sprite_from_spec(map)
                .map_err(mlua::Error::runtime)?;
            Ok(LuaSprite::new(handle, builder_for_sprite.clone()))
        })?,
    )?;

    let builder_for_text = builder.clone();
    sb.set(
        "text",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            let handle = builder_for_text
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .create_text_from_spec(map)
                .map_err(mlua::Error::runtime)?;
            Ok(LuaTimelineObject::new(
                TimelineObjectHandle::Text(handle),
                builder_for_text.clone(),
            ))
        })?,
    )?;

    let builder_for_video = builder.clone();
    sb.set(
        "video",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            let handle = builder_for_video
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .create_video_from_spec(map)
                .map_err(mlua::Error::runtime)?;
            Ok(LuaTimelineObject::new(
                TimelineObjectHandle::Video(handle),
                builder_for_video.clone(),
            ))
        })?,
    )?;

    let builder_for_line = builder.clone();
    sb.set(
        "line",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            let handle = builder_for_line
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .create_line_from_spec(map)
                .map_err(mlua::Error::runtime)?;
            Ok(LuaTimelineObject::new(
                TimelineObjectHandle::Line(handle),
                builder_for_line.clone(),
            ))
        })?,
    )?;

    let builder_for_controller = builder.clone();
    sb.set(
        "controller",
        lua.create_function(move |lua, spec: Table| -> mlua::Result<Value> {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            if map.contains_key("states") || map.contains_key("States") {
                builder_for_controller
                    .lock()
                    .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                    .create_controller_from_spec(map);
                return Ok(Value::Nil);
            }
            let controller_id = controller_handle_id(&map);
            if map.is_empty() || controller_id.is_some() {
                let handle = builder_for_controller
                    .lock()
                    .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                    .create_controller_from_spec(map);
                let ud = lua
                    .create_userdata(LuaController::new(handle, builder_for_controller.clone()))?;
                return Ok(Value::UserData(ud));
            }

            builder_for_controller
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_controller_inline(lua_table_to_inline_controller(map));
            Ok(Value::Nil)
        })?,
    )?;

    let builder_for_note = builder.clone();
    sb.set(
        "note_controller",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            builder_for_note
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_note_controller(map);
            Ok(())
        })?,
    )?;

    let builder_for_trigger = builder.clone();
    sb.set(
        "trigger",
        lua.create_function(move |lua, spec: Table| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            builder_for_trigger
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .push_trigger(map);
            Ok(())
        })?,
    )?;

    let builder_for_template = builder.clone();
    sb.set(
        "template",
        lua.create_function(move |lua, (name, spec): (String, Table)| {
            let map = table_to_authoring_map(lua, Value::Table(spec))?;
            builder_for_template
                .lock()
                .map_err(|_| mlua::Error::runtime("sb builder lock poisoned"))?
                .set_template(name, map);
            Ok(())
        })?,
    )?;

    sb.set("null", Value::UserData(lua.create_userdata(NullMarker)?))?;

    env.set("sb", sb)?;
    Ok(())
}

pub(crate) fn table_to_authoring_map(
    lua: &Lua,
    value: Value,
) -> mlua::Result<Map<String, JsonValue>> {
    match value {
        Value::Table(table) => {
            let map = match lua_value_to_json(lua, Value::Table(table))? {
                JsonValue::Object(map) => map,
                _ => {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: "table",
                        to: "object".into(),
                        message: Some("expected object/table".into()),
                    });
                }
            };
            Ok(lua_table_to_authoring_map(map))
        }
        _ => Err(mlua::Error::FromLuaConversionError {
            from: "table",
            to: "object".into(),
            message: Some("expected object/table".into()),
        }),
    }
}

fn lua_key_to_string(key: Value) -> mlua::Result<String> {
    match key {
        Value::String(s) => Ok(s.to_str()?.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(mlua::Error::runtime("table key must be string or number")),
    }
}

fn lua_table_to_json_map(lua: &Lua, table: Table) -> mlua::Result<Map<String, JsonValue>> {
    let mut map = Map::new();
    for pair in table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        map.insert(lua_key_to_string(key)?, lua_value_to_json(lua, value)?);
    }
    Ok(map)
}

fn lua_value_to_json(lua: &Lua, value: Value) -> mlua::Result<JsonValue> {
    match value {
        Value::Nil => Ok(JsonValue::Null),
        Value::UserData(ud) if ud.is::<NullMarker>() => Ok(JsonValue::Null),
        Value::Boolean(b) => Ok(JsonValue::Bool(b)),
        Value::Integer(i) => Ok(json!(i)),
        Value::Number(n) => Ok(json!(n)),
        Value::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        Value::Table(table) => {
            let seq: Vec<JsonValue> = table
                .sequence_values::<Value>()
                .map(|value| lua_value_to_json(lua, value?))
                .collect::<mlua::Result<_>>()?;
            if !seq.is_empty() {
                Ok(JsonValue::Array(seq))
            } else {
                Ok(JsonValue::Object(lua_table_to_json_map(lua, table)?))
            }
        }
        other => lua.from_value(other),
    }
}

fn controller_handle_id(map: &Map<String, JsonValue>) -> Option<String> {
    map.get("id")
        .or_else(|| map.get("Id"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn compiles_sprite_script() {
        let source = r#"
local glow = sb.sprite { path = "hello_glow.png", id = "hello_glow" }
glow:key(0.0, { opacity = 0, scale = 0.8 })
glow:key(1.0, { opacity = 1, scale = 1.0, ease = "out_quint" })
"#;
        let doc = compile_source("storyboard.lua", source).unwrap();
        assert_eq!(doc.sprites.len(), 1);
        assert_eq!(doc.sprites[0]["id"], "hello_glow");
        assert_eq!(doc.sprites[0]["states"].as_array().unwrap().len(), 2);
        assert_eq!(doc.sprites[0]["states"][1]["easing"], "out_quint");
    }

    #[test]
    fn compiles_kou_ppap_shape() {
        let source = r##"
sb.controller { scanline_opacity = 0, time = 0 }
local c2 = sb.controller {}
c2:key(0, {})
c2:rel(0.1, {
  note_ring_color = "#FFFFFF",
  note_fill_colors = {
    "#d7a743", "#d7a743", "#d7a743", "#d7a743",
    "#d7a743", "#d7a743", "#d7a743", "#d7a743",
    "#d7a743", "#d7a743", "#d7a743", "#d7a743",
  },
})
local s = sb.sprite {
  path = "ppap1.png",
  width = 800,
  height = 600,
  layer = 0,
  opacity = 1,
  time = 0.1,
  easing = "none",
}
s:key(10000, { opacity = 1, destroy = true })
"##;
        let doc = compile_source("kou.ppap.lua", source).unwrap();
        assert_eq!(doc.controllers.len(), 2);
        assert_eq!(doc.sprites.len(), 1);
        assert_eq!(doc.controllers[0]["scanline_opacity"], 0);
        assert_eq!(doc.sprites[0]["path"], "ppap1.png");
    }

    #[test]
    fn compiles_controller_script() {
        let source = r#"
local cam = sb.controller {}
cam:key(0.0, {
  background_dim = 1,
  note_opacity_multiplier = 0,
  scanline_opacity = 0,
  ui_opacity = 0,
})
"#;
        let doc = compile_source("controller.lua", source).unwrap();
        assert_eq!(doc.controllers.len(), 1);
        let state = &doc.controllers[0]["states"].as_array().unwrap()[0];
        assert_eq!(state["background_dim"], 1.0);
    }

    #[test]
    fn controller_with_id_returns_handle() {
        let source = r#"
local cam = sb.controller { id = "cam" }
cam:key(0.0, { ui_opacity = 1 })
"#;
        let doc = compile_source("controller-id.lua", source).unwrap();
        assert_eq!(doc.controllers.len(), 1);
        assert_eq!(doc.controllers[0]["id"], "cam");
        assert_eq!(doc.controllers[0]["states"][0]["ui_opacity"], 1.0);
    }

    #[test]
    fn compiles_note_controller_loop() {
        let source = r#"
for _, nc in ipairs({ { note = 422, y = 0.0 }, { note = 423, y = 0.75 } }) do
  sb.note_controller {
    override_y = true,
    y = nc.y,
    time = 0.0,
    note = nc.note,
  }
end
"#;
        let doc = compile_source("nc.lua", source).unwrap();
        assert_eq!(doc.note_controllers.len(), 2);
        assert_eq!(doc.note_controllers[0]["note"], 422);
    }

    #[test]
    fn compiles_stage_objects_and_trigger() {
        let source = r#"
local title = sb.text { id = "title", text = "READY", time = 0, x = 0, y = 120 }
title:key(1.0, { opacity = 1, text = "GO" })

local movie = sb.video { id = "mv", path = "intro.mp4", time = 0, opacity = 0 }
movie:key(2.0, { opacity = 1 })

local rail = sb.line {
  id = "rail",
  pos = {
    { x = -0.5, y = 0.0 },
    { x = 0.5, y = 0.0 },
  },
  time = 0,
}
rail:key(1.0, { opacity = 0.5 })

sb.trigger { type = "NoteClear", notes = { 1 }, spawn = { "title" } }
"#;
        let doc = compile_source("objects.lua", source).unwrap();
        assert_eq!(doc.texts.len(), 1);
        assert_eq!(doc.videos.len(), 1);
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.triggers.len(), 1);
        assert_eq!(doc.texts[0]["states"][0]["text"], "GO");
        assert_eq!(doc.videos[0]["path"], "intro.mp4");
        assert_eq!(doc.triggers[0]["type"], "NoteClear");
    }

    #[test]
    fn sandbox_blocks_os() {
        let err = compile_source("evil.lua", "os.execute('echo pwned')").unwrap_err();
        assert!(err.to_string().contains("not available in sandbox"));
    }

    #[test]
    fn sandbox_blocks_load() {
        let err = compile_source("evil.lua", "load('bad')").unwrap_err();
        assert!(err.to_string().contains("not available in sandbox"));
    }

    #[test]
    fn preserves_null_fields_from_lua() {
        let source = r#"
sb.controller {
  states = {
    { scanline_color = sb.null, time = 1.0 },
  },
}
"#;
        let doc = compile_source("null.lua", source).unwrap();
        let state = doc.controllers[0]["states"][0].as_object().unwrap();
        assert!(state.contains_key("scanline_color"));
        assert!(state["scanline_color"].is_null());
    }

    #[test]
    fn compiles_example_file() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/hello/storyboard.lua");
        if !path.exists() {
            return;
        }
        let doc = compile_file(&path).unwrap();
        assert_eq!(doc.sprites.len(), 1);
        assert_eq!(doc.sprites[0]["path"], "hello_glow.png");
    }
}
