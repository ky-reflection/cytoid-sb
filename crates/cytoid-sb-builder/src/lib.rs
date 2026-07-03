use cytoid_sb_model::StoryboardDocument;
use serde_json::{json, Map, Value};

/// JSON shape for Lua-authored storyboards (Unity uncompiled) vs compiled PascalCase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Authoring,
    Compiled,
}

/// Index into [`StoryboardBuilder`] sprite list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteHandle {
    pub index: usize,
}

/// Index into [`StoryboardBuilder`] text list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextHandle {
    pub index: usize,
}

/// Index into [`StoryboardBuilder`] video list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoHandle {
    pub index: usize,
}

/// Index into [`StoryboardBuilder`] line list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineHandle {
    pub index: usize,
}

/// Index into [`StoryboardBuilder`] controller list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerHandle {
    pub index: usize,
}

/// Collects authoring calls from Lua/TS/Rust frontends into a [`StoryboardDocument`].
#[derive(Debug)]
pub struct StoryboardBuilder {
    doc: StoryboardDocument,
    format: OutputFormat,
}

impl Default for StoryboardBuilder {
    fn default() -> Self {
        Self {
            doc: StoryboardDocument::default(),
            format: OutputFormat::Authoring,
        }
    }
}

impl StoryboardBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_format(format: OutputFormat) -> Self {
        Self {
            doc: StoryboardDocument::default(),
            format,
        }
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }

    pub fn push_controller_inline(&mut self, mut spec: Map<String, Value>) {
        spec.remove("id");
        spec.remove("Id");
        let obj = match self.format {
            OutputFormat::Authoring => normalize_authoring_object_keys(spec),
            OutputFormat::Compiled => normalize_object_keys(spec),
        };
        self.doc.controllers.push(Value::Object(obj));
    }

    pub fn push_note_controller(&mut self, spec: Map<String, Value>) {
        let obj = match self.format {
            OutputFormat::Authoring => normalize_authoring_object_keys(spec),
            OutputFormat::Compiled => normalize_object_keys(spec),
        };
        self.doc.note_controllers.push(Value::Object(obj));
    }

    pub fn push_trigger(&mut self, spec: Map<String, Value>) {
        let obj = match self.format {
            OutputFormat::Authoring => normalize_authoring_object_keys(spec),
            OutputFormat::Compiled => normalize_object_keys(spec),
        };
        self.doc.triggers.push(Value::Object(obj));
    }

    pub fn create_sprite_from_spec(
        &mut self,
        spec: Map<String, Value>,
    ) -> Result<SpriteHandle, String> {
        create_stage_object(
            &mut self.doc.sprites,
            self.format,
            "sprite",
            spec,
            RequiredField::Path,
        )
        .map(|index| SpriteHandle { index })
    }

    pub fn create_text_from_spec(
        &mut self,
        spec: Map<String, Value>,
    ) -> Result<TextHandle, String> {
        create_stage_object(
            &mut self.doc.texts,
            self.format,
            "text",
            spec,
            RequiredField::Text,
        )
        .map(|index| TextHandle { index })
    }

    pub fn create_video_from_spec(
        &mut self,
        spec: Map<String, Value>,
    ) -> Result<VideoHandle, String> {
        create_stage_object(
            &mut self.doc.videos,
            self.format,
            "video",
            spec,
            RequiredField::Path,
        )
        .map(|index| VideoHandle { index })
    }

    pub fn create_line_from_spec(
        &mut self,
        spec: Map<String, Value>,
    ) -> Result<LineHandle, String> {
        create_stage_object(
            &mut self.doc.lines,
            self.format,
            "line",
            spec,
            RequiredField::PosOrState,
        )
        .map(|index| LineHandle { index })
    }

    pub fn push_text_keyframe(
        &mut self,
        handle: TextHandle,
        time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.texts,
            self.format,
            handle.index,
            "text",
            Some(time),
            None,
            patch,
        )
    }

    pub fn push_text_relative_keyframe(
        &mut self,
        handle: TextHandle,
        add_time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.texts,
            self.format,
            handle.index,
            "text",
            None,
            Some(add_time),
            patch,
        )
    }

    pub fn push_video_keyframe(
        &mut self,
        handle: VideoHandle,
        time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.videos,
            self.format,
            handle.index,
            "video",
            Some(time),
            None,
            patch,
        )
    }

    pub fn push_video_relative_keyframe(
        &mut self,
        handle: VideoHandle,
        add_time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.videos,
            self.format,
            handle.index,
            "video",
            None,
            Some(add_time),
            patch,
        )
    }

    pub fn push_line_keyframe(
        &mut self,
        handle: LineHandle,
        time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.lines,
            self.format,
            handle.index,
            "line",
            Some(time),
            None,
            patch,
        )
    }

    pub fn push_line_relative_keyframe(
        &mut self,
        handle: LineHandle,
        add_time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.lines,
            self.format,
            handle.index,
            "line",
            None,
            Some(add_time),
            patch,
        )
    }

    pub fn create_controller(&mut self, id: Option<String>) -> ControllerHandle {
        let mut spec = Map::new();
        if let Some(id) = id {
            spec.insert("id".into(), json!(id));
        }
        self.create_controller_from_spec(spec)
    }

    pub fn create_controller_from_spec(
        &mut self,
        mut spec: Map<String, Value>,
    ) -> ControllerHandle {
        let index = self.doc.controllers.len();
        let mut obj = Map::new();
        if let Some(id) = spec.remove("id").or_else(|| spec.remove("Id")) {
            obj.insert(field_name(self.format, "id"), id);
        }
        let states = spec
            .remove("states")
            .or_else(|| spec.remove("States"))
            .unwrap_or_else(|| Value::Array(Vec::new()));
        for (key, value) in spec {
            obj.insert(normalize_root_key(self.format, &key), value);
        }
        obj.insert(states_key(self.format), states);
        self.doc.controllers.push(Value::Object(obj));
        ControllerHandle { index }
    }

    pub fn push_sprite_keyframe(
        &mut self,
        handle: SpriteHandle,
        time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.sprites,
            self.format,
            handle.index,
            "sprite",
            Some(time),
            None,
            patch,
        )
    }

    pub fn push_sprite_relative_keyframe(
        &mut self,
        handle: SpriteHandle,
        add_time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.sprites,
            self.format,
            handle.index,
            "sprite",
            None,
            Some(add_time),
            patch,
        )
    }

    pub fn push_controller_keyframe(
        &mut self,
        handle: ControllerHandle,
        time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.controllers,
            self.format,
            handle.index,
            "controller",
            Some(time),
            None,
            patch,
        )
    }

    pub fn push_controller_relative_keyframe(
        &mut self,
        handle: ControllerHandle,
        add_time: f64,
        patch: Value,
    ) -> Result<(), String> {
        push_timed_keyframe(
            &mut self.doc.controllers,
            self.format,
            handle.index,
            "controller",
            None,
            Some(add_time),
            patch,
        )
    }

    /// Compiled-format helper retained for tests.
    pub fn add_sprite(
        &mut self,
        id: impl Into<String>,
        path: impl Into<String>,
        base_state: Value,
    ) -> &mut Self {
        let mut builder = StoryboardBuilder::with_format(OutputFormat::Compiled);
        builder.doc = std::mem::take(&mut self.doc);
        builder.add_sprite_compiled(id, path, base_state);
        self.doc = builder.doc;
        self
    }

    fn add_sprite_compiled(
        &mut self,
        id: impl Into<String>,
        path: impl Into<String>,
        base_state: Value,
    ) -> &mut Self {
        let path = path.into();
        let mut state = ensure_time_key(OutputFormat::Compiled, base_state);
        if let Value::Object(map) = &mut state {
            map.entry("Path").or_insert_with(|| json!(path));
        }
        self.doc.sprites.push(json!({
            "Id": id.into(),
            "States": [state],
        }));
        self
    }

    pub fn finish(self) -> StoryboardDocument {
        self.doc
    }

    pub fn snapshot(&self) -> StoryboardDocument {
        self.doc.clone()
    }
}

fn states_key(format: OutputFormat) -> String {
    match format {
        OutputFormat::Authoring => "states".into(),
        OutputFormat::Compiled => "States".into(),
    }
}

fn field_name(format: OutputFormat, snake: &str) -> String {
    match format {
        OutputFormat::Authoring => snake.to_string(),
        OutputFormat::Compiled => snake_to_pascal(snake),
    }
}

fn normalize_root_key(format: OutputFormat, key: &str) -> String {
    match format {
        OutputFormat::Authoring => lua_field_to_authoring_json(key),
        OutputFormat::Compiled => lua_field_to_json(key),
    }
}

#[derive(Debug, Clone, Copy)]
enum RequiredField {
    Path,
    Text,
    PosOrState,
}

fn create_stage_object(
    entries: &mut Vec<Value>,
    format: OutputFormat,
    kind: &str,
    mut spec: Map<String, Value>,
    required: RequiredField,
) -> Result<usize, String> {
    if !has_required_stage_field(&spec, required) {
        return Err(match required {
            RequiredField::Path => format!("{kind} requires path"),
            RequiredField::Text => format!("{kind} requires text"),
            RequiredField::PosOrState => format!("{kind} requires pos or states"),
        });
    }

    let index = entries.len();
    let mut obj = Map::new();
    if let Some(id) = spec.remove("id").or_else(|| spec.remove("Id")) {
        obj.insert(field_name(format, "id"), id);
    }
    let states = spec
        .remove("states")
        .or_else(|| spec.remove("States"))
        .unwrap_or_else(|| Value::Array(Vec::new()));
    for (k, v) in spec {
        obj.insert(normalize_root_key(format, &k), v);
    }
    obj.insert(states_key(format), states);
    entries.push(Value::Object(obj));
    Ok(index)
}

fn has_required_stage_field(spec: &Map<String, Value>, required: RequiredField) -> bool {
    match required {
        RequiredField::Path => spec.contains_key("path") || spec.contains_key("Path"),
        RequiredField::Text => spec.contains_key("text") || spec.contains_key("Text"),
        RequiredField::PosOrState => {
            spec.contains_key("pos")
                || spec.contains_key("Pos")
                || spec.contains_key("states")
                || spec.contains_key("States")
        }
    }
}

fn push_timed_keyframe(
    entries: &mut [Value],
    format: OutputFormat,
    index: usize,
    kind: &str,
    time: Option<f64>,
    add_time: Option<f64>,
    patch: Value,
) -> Result<(), String> {
    let entry = entries
        .get_mut(index)
        .ok_or_else(|| format!("internal error: invalid {kind} handle {index}"))?;

    let obj = entry
        .as_object_mut()
        .ok_or_else(|| format!("{kind} entry is not an object"))?;
    let states = obj
        .entry(states_key(format))
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(states_arr) = states.as_array_mut() else {
        return Err(format!("{kind} states is not an array"));
    };

    let mut state = match patch {
        Value::Object(map) => normalize_state_patch(format, map),
        _ => return Err("keyframe patch must be a table/object".into()),
    };
    if let Some(time) = time {
        state.insert(field_name(format, "time"), json!(time));
    }
    if let Some(add_time) = add_time {
        state.insert(field_name(format, "add_time"), json!(add_time));
    }
    states_arr.push(Value::Object(state));
    Ok(())
}

fn normalize_state_patch(format: OutputFormat, patch: Map<String, Value>) -> Map<String, Value> {
    match format {
        OutputFormat::Authoring => normalize_authoring_object_keys(patch),
        OutputFormat::Compiled => normalize_keyframe_patch(patch),
    }
}

fn ensure_time_key(format: OutputFormat, base_state: Value) -> Value {
    let time_key = field_name(format, "time");
    match base_state {
        Value::Object(mut map) => {
            if !map.contains_key(&time_key) {
                map.insert(time_key, json!(0.0));
            }
            Value::Object(map)
        }
        other => {
            let mut map = Map::new();
            map.insert(time_key, json!(0.0));
            map.insert("_".into(), other);
            Value::Object(map)
        }
    }
}

/// Convenience helper used by tests.
pub fn sprite_pulse(id: &str, path: &str, time: f64, opacity: f64) -> StoryboardBuilder {
    let mut builder = StoryboardBuilder::with_format(OutputFormat::Compiled);
    builder.add_sprite_compiled(
        id,
        path,
        json!({
            "Time": time,
            "Opacity": opacity,
        }),
    );
    builder
}

/// Normalize Lua-style snake_case patch keys to compiled Cytoid JSON PascalCase.
pub fn normalize_keyframe_patch(patch: Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in patch {
        if key == "time" || key == "add_time" {
            continue;
        }
        let json_key = lua_field_to_json(&key);
        let json_value = if key == "ease" || key == "easing" {
            json!(normalize_easing(value))
        } else {
            value
        };
        out.insert(json_key, json_value);
    }
    out
}

/// Convert a Lua spec table to an authoring object map (snake_case keys preserved).
pub fn lua_table_to_authoring_map(spec: Map<String, Value>) -> Map<String, Value> {
    spec
}

pub fn lua_table_to_inline_controller(spec: Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in spec {
        if key == "id" || key == "Id" {
            continue;
        }
        out.insert(key, value);
    }
    out
}

fn normalize_object_keys(spec: Map<String, Value>) -> Map<String, Value> {
    spec.into_iter()
        .map(|(key, value)| (lua_field_to_json(&key), value))
        .collect()
}

fn normalize_authoring_object_keys(spec: Map<String, Value>) -> Map<String, Value> {
    spec.into_iter()
        .map(|(key, value)| (lua_field_to_authoring_json(&key), value))
        .collect()
}

fn lua_field_to_authoring_json(key: &str) -> String {
    match key {
        "ease" => "easing".into(),
        other if other.chars().next().is_some_and(|c| c.is_ascii_uppercase()) => {
            pascal_to_snake(other)
        }
        other => other.to_string(),
    }
}

fn lua_field_to_json(key: &str) -> String {
    match key {
        "opacity" => "Opacity".into(),
        "scale" => "Scale".into(),
        "scale_x" => "ScaleX".into(),
        "scale_y" => "ScaleY".into(),
        "x" => "X".into(),
        "y" => "Y".into(),
        "z" => "Z".into(),
        "rot_x" => "RotX".into(),
        "rot_y" => "RotY".into(),
        "rot_z" => "RotZ".into(),
        "ease" | "easing" => "Easing".into(),
        "layer" => "Layer".into(),
        "order" => "Order".into(),
        "destroy" => "Destroy".into(),
        "background_dim" => "BackgroundDim".into(),
        "note_opacity_multiplier" => "NoteOpacityMultiplier".into(),
        "scanline_opacity" => "ScanlineOpacity".into(),
        "ui_opacity" => "UiOpacity".into(),
        "storyboard_opacity" => "StoryboardOpacity".into(),
        "note_ring_color" => "NoteRingColor".into(),
        "note_fill_colors" => "NoteFillColors".into(),
        "scanline_color" => "ScanlineColor".into(),
        "override_scanline_pos" => "OverrideScanlinePos".into(),
        "scanline_pos" => "ScanlinePos".into(),
        "override_y" => "OverrideY".into(),
        "note" => "Note".into(),
        other if other.contains('_') => snake_to_pascal(other),
        other if other.chars().next().is_some_and(|c| c.is_ascii_lowercase()) => {
            snake_to_pascal(other)
        }
        other => other.to_string(),
    }
}

fn snake_to_pascal(key: &str) -> String {
    key.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn pascal_to_snake(key: &str) -> String {
    let mut out = String::new();
    let mut prev_lower_or_digit = false;
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower_or_digit = false;
        } else {
            out.push(c);
            prev_lower_or_digit = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    out
}

fn normalize_easing(value: Value) -> String {
    let raw = match value {
        Value::String(s) => s,
        other => return other.to_string(),
    };
    match raw.to_ascii_lowercase().as_str() {
        "linear" | "none" => "Linear".into(),
        "out_quint" | "outquint" => "OutQuint".into(),
        "in_quint" | "inquint" => "InQuint".into(),
        "in_out_quint" | "inoutquint" => "InOutQuint".into(),
        other => snake_to_pascal(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_sprite_document() {
        let doc = sprite_pulse("glow", "glow.png", 0.0, 1.0).finish();
        assert_eq!(doc.sprites.len(), 1);
        assert_eq!(doc.sprites[0]["States"][0]["Path"], "glow.png");
    }

    #[test]
    fn authoring_sprite_spec_and_keyframe() {
        let mut builder = StoryboardBuilder::new();
        let mut spec = Map::new();
        spec.insert("path".into(), json!("ppap1.png"));
        spec.insert("width".into(), json!(800));
        spec.insert("time".into(), json!(0.1));
        let handle = builder.create_sprite_from_spec(spec).unwrap();
        builder
            .push_sprite_keyframe(handle, 10000.0, json!({"opacity": 1, "destroy": true}))
            .unwrap();
        let doc = builder.finish();
        assert_eq!(doc.sprites[0]["path"], "ppap1.png");
        assert_eq!(doc.sprites[0]["width"], 800);
        assert_eq!(doc.sprites[0]["states"][0]["destroy"], true);
    }

    #[test]
    fn authoring_normalizes_pascal_and_ease_aliases() {
        let mut builder = StoryboardBuilder::new();
        let mut spec = Map::new();
        spec.insert("Path".into(), json!("a.png"));
        let handle = builder.create_sprite_from_spec(spec).unwrap();
        builder
            .push_sprite_keyframe(handle, 0.0, json!({"Opacity": 1, "ease": "out_quint"}))
            .unwrap();
        let doc = builder.finish();
        assert_eq!(doc.sprites[0]["path"], "a.png");
        assert_eq!(doc.sprites[0]["states"][0]["opacity"], 1);
        assert_eq!(doc.sprites[0]["states"][0]["easing"], "out_quint");
    }

    #[test]
    fn compiled_note_controller_keys() {
        let mut builder = StoryboardBuilder::with_format(OutputFormat::Compiled);
        builder.push_note_controller(Map::from_iter([
            ("override_y".into(), json!(true)),
            ("note".into(), json!(422)),
        ]));
        let doc = builder.snapshot();
        assert_eq!(doc.note_controllers[0]["OverrideY"], true);
        assert_eq!(doc.note_controllers[0]["Note"], 422);
    }

    #[test]
    fn authoring_controller_relative_keyframe() {
        let mut builder = StoryboardBuilder::new();
        let handle = builder.create_controller(None);
        builder
            .push_controller_keyframe(handle, 0.0, json!({}))
            .unwrap();
        builder
            .push_controller_relative_keyframe(handle, 0.1, json!({"note_ring_color": "#FFFFFF"}))
            .unwrap();
        let doc = builder.snapshot();
        let states = doc.controllers[0]["states"].as_array().unwrap();
        assert_eq!(states.len(), 2);
        assert_eq!(states[1]["add_time"], 0.1);
    }
}
