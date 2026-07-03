use cytoid_sb_model::{
    field::{
        has_id_target_conflict, has_root_or_state_content, has_target_parent_conflict,
        object_field, object_field_str, object_id, object_states, template_ref,
    },
    StoryboardDocument,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn push(&mut self, code: &'static str, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            code,
            message: message.into(),
        });
    }
}

pub fn validate_document(doc: &StoryboardDocument) -> ValidationReport {
    let mut report = ValidationReport::default();

    if !doc.has_content() {
        report.push("empty_document", "storyboard has no objects or triggers");
    }

    let template_ids: HashSet<&str> = doc
        .templates
        .as_ref()
        .map(|m| m.keys().map(String::as_str).collect())
        .unwrap_or_default();

    validate_collection(
        &mut report,
        "sprite",
        &doc.sprites,
        &template_ids,
        validate_sprite,
    );
    validate_collection(
        &mut report,
        "text",
        &doc.texts,
        &template_ids,
        validate_text,
    );
    validate_collection(
        &mut report,
        "video",
        &doc.videos,
        &template_ids,
        validate_video,
    );
    validate_collection(
        &mut report,
        "line",
        &doc.lines,
        &template_ids,
        validate_line,
    );
    validate_collection(
        &mut report,
        "controller",
        &doc.controllers,
        &template_ids,
        validate_controller,
    );
    validate_collection(
        &mut report,
        "note_controller",
        &doc.note_controllers,
        &template_ids,
        validate_note_controller,
    );
    validate_triggers(&mut report, &doc.triggers);

    report
}

type ObjectValidator = fn(&mut ValidationReport, &str, &serde_json::Value, &HashSet<&str>);

fn validate_collection(
    report: &mut ValidationReport,
    kind: &str,
    entries: &[serde_json::Value],
    template_ids: &HashSet<&str>,
    validate: ObjectValidator,
) {
    let mut seen_ids = HashMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let label = object_id(entry).unwrap_or_else(|| format!("#{index}"));
        validate_object_identity(report, kind, &label, entry, template_ids);
        if let Some(id) = object_id(entry) {
            if let Some(prev) = seen_ids.insert(id.clone(), kind) {
                report.push(
                    "duplicate_id",
                    format!("duplicate {kind} id '{id}' (also used by {prev})"),
                );
            }
        }
        validate(report, kind, entry, template_ids);
    }
}

fn validate_object_identity(
    report: &mut ValidationReport,
    kind: &str,
    label: &str,
    value: &serde_json::Value,
    template_ids: &HashSet<&str>,
) {
    if has_id_target_conflict(value) {
        report.push(
            "id_target_conflict",
            format!("{kind} {label} has both id and target_id"),
        );
    }
    if has_target_parent_conflict(value) {
        report.push(
            "target_parent_conflict",
            format!("{kind} {label} has both target_id and parent_id"),
        );
    }
    if let Some(template) = template_ref(value) {
        if !template_ids.contains(template.as_str()) {
            report.push(
                "unknown_template",
                format!("{kind} {label} references unknown template '{template}'"),
            );
        }
    }
}

fn validate_sprite(
    report: &mut ValidationReport,
    kind: &str,
    value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    let label = object_id(value).unwrap_or_else(|| kind.to_string());
    let path = object_field_str(value, &["Path", "path"]);
    let states = object_states(value);
    if path.is_none() && states.is_empty() && !has_root_or_state_content(value) {
        report.push(
            "sprite_missing_content",
            format!("sprite {label} has no path, states, or root time"),
        );
    }
}

fn validate_text(
    report: &mut ValidationReport,
    kind: &str,
    value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    let label = object_id(value).unwrap_or_else(|| kind.to_string());
    let text = object_field_str(value, &["text", "Text"]);
    let states = object_states(value);
    if text.is_none() && states.is_empty() && !has_root_or_state_content(value) {
        report.push(
            "text_missing_content",
            format!("text {label} has no text, states, or root time"),
        );
    }
}

fn validate_video(
    report: &mut ValidationReport,
    kind: &str,
    value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    let label = object_id(value).unwrap_or_else(|| kind.to_string());
    let path = object_field_str(value, &["Path", "path"]);
    let states = object_states(value);
    if path.is_none() && states.is_empty() && !has_root_or_state_content(value) {
        report.push(
            "video_missing_content",
            format!("video {label} has no path, states, or root time"),
        );
    }
}

fn validate_line(
    report: &mut ValidationReport,
    kind: &str,
    value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    let label = object_id(value).unwrap_or_else(|| kind.to_string());
    let pos = object_field(value, &["pos", "Pos"]);
    let states = object_states(value);
    if pos.is_none() && states.is_empty() && !has_root_or_state_content(value) {
        report.push(
            "line_missing_content",
            format!("line {label} has no pos, states, or root time"),
        );
    }
}

fn validate_controller(
    _report: &mut ValidationReport,
    _kind: &str,
    _value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    // Controllers may be anonymous (no id) with only root-level fields — always valid if present.
}

fn validate_note_controller(
    report: &mut ValidationReport,
    kind: &str,
    value: &serde_json::Value,
    _templates: &HashSet<&str>,
) {
    let label = object_id(value).unwrap_or_else(|| kind.to_string());
    if !has_note_binding(value) {
        report.push(
            "note_controller_missing_note",
            format!("note_controller {label} has no note binding"),
        );
    }
}

fn has_note_binding(value: &serde_json::Value) -> bool {
    if object_field(value, &["note", "Note"]).is_some() {
        return true;
    }
    object_states(value)
        .iter()
        .any(|state| object_field(state, &["note", "Note"]).is_some())
}

fn validate_triggers(report: &mut ValidationReport, triggers: &[serde_json::Value]) {
    for (index, trigger) in triggers.iter().enumerate() {
        let label = format!("#{}", index);
        let trigger_type =
            object_field_str(trigger, &["type", "Type"]).unwrap_or_else(|| "None".into());
        let has_event = matches!(
            trigger_type.to_ascii_lowercase().as_str(),
            "noteclear" | "combo" | "score"
        ) || object_field(trigger, &["notes", "Notes"]).is_some()
            || object_field(trigger, &["combo", "Combo"]).is_some()
            || object_field(trigger, &["score", "Score"]).is_some();

        if !has_event {
            report.push(
                "trigger_missing_event",
                format!("trigger {label} has no type (NoteClear/Combo/Score) or matching field"),
            );
        }

        let spawn = object_field(trigger, &["spawn", "Spawn"]);
        let destroy = object_field(trigger, &["destroy", "Destroy"]);
        let has_action = spawn
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty())
            || destroy
                .and_then(|v| v.as_array())
                .is_some_and(|a| !a.is_empty());

        if !has_action {
            report.push(
                "trigger_missing_action",
                format!("trigger {label} has empty spawn and destroy lists"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_lab_style_sprite() {
        let doc = StoryboardDocument {
            sprites: vec![json!({"path": "a.png", "states": [{"time": 0.0}]})],
            ..Default::default()
        };
        assert!(validate_document(&doc).is_ok());
    }

    #[test]
    fn accepts_controller_only_like_offset_guide() {
        let doc = StoryboardDocument {
            controllers: vec![json!({
                "time": 0,
                "background_dim": 1,
                "note_opacity_multiplier": 0,
                "scanline_opacity": 0,
                "ui_opacity": 0
            })],
            ..Default::default()
        };
        assert!(validate_document(&doc).is_ok());
    }

    #[test]
    fn accepts_kou_ppap_shape() {
        let doc = StoryboardDocument {
            controllers: vec![
                json!({"scanline_opacity": 0, "time": 0}),
                json!({"states": [{"time": 0}, {"note_ring_color": "#FFFFFF", "add_time": 0.1}]}),
            ],
            sprites: vec![json!({
                "path": "ppap1.png",
                "time": 0.1,
                "states": [{"opacity": 1, "time": 10000, "destroy": true}]
            })],
            ..Default::default()
        };
        assert!(validate_document(&doc).is_ok());
    }

    #[test]
    fn flags_empty_document() {
        assert!(!validate_document(&StoryboardDocument::default()).is_ok());
    }

    #[test]
    fn flags_id_target_conflict() {
        let doc = StoryboardDocument {
            sprites: vec![json!({"id": "a", "target_id": "b", "path": "x.png"})],
            ..Default::default()
        };
        let report = validate_document(&doc);
        assert!(report.issues.iter().any(|i| i.code == "id_target_conflict"));
    }

    #[test]
    fn flags_unknown_template() {
        let doc = StoryboardDocument {
            sprites: vec![json!({"template": "missing", "path": "a.png"})],
            ..Default::default()
        };
        let report = validate_document(&doc);
        assert!(report.issues.iter().any(|i| i.code == "unknown_template"));
    }

    #[test]
    fn validates_trigger_type_field() {
        let doc = StoryboardDocument {
            triggers: vec![json!({
                "type": "NoteClear",
                "notes": [1, 2],
                "spawn": ["fx"]
            })],
            ..Default::default()
        };
        assert!(validate_document(&doc).is_ok());
    }
}
