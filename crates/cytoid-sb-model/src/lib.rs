use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub mod field;
pub mod summary;

pub use field::{
    has_id_target_conflict, has_root_or_state_content, has_target_parent_conflict, object_field,
    object_field_str, object_id, object_states, template_ref,
};
pub use summary::DocumentSummary;

/// Top-level storyboard document matching Unity `Storyboard.cs` JSON keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StoryboardDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub templates: Option<Map<String, Value>>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub texts: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sprites: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub videos: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controllers: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub note_controllers: Vec<Value>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<Value>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub compiled: bool,
}

impl StoryboardDocument {
    pub fn object_count(&self) -> usize {
        self.texts.len()
            + self.sprites.len()
            + self.videos.len()
            + self.lines.len()
            + self.controllers.len()
            + self.note_controllers.len()
    }

    pub fn has_content(&self) -> bool {
        self.object_count() > 0 || !self.triggers.is_empty()
    }

    pub fn summary(&self) -> DocumentSummary {
        DocumentSummary::from(self)
    }

    /// Parse storyboard JSON text (strict JSON or JSON5 with `//` / `/* */` comments).
    pub fn from_json_text(text: &str) -> Result<Self, serde_json::Error> {
        match serde_json::from_str(text) {
            Ok(doc) => Ok(doc),
            Err(first) => json5::from_str(text).map_err(|_| first),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unity_compiled_document() {
        let json = r#"{
            "compiled": true,
            "sprites": [{
                "Id": "glow",
                "Path": "glow.png",
                "States": [{ "Time": 0.0, "Opacity": 0.0 }]
            }]
        }"#;
        let doc: StoryboardDocument = serde_json::from_str(json).expect("parse");
        assert_eq!(doc.sprites.len(), 1);
        assert!(doc.compiled);
    }

    #[test]
    fn parses_lab_uncompiled_snippet() {
        let json = r#"{
            "controllers": [{ "time": 0 }],
            "sprites": [{ "path": "a.png", "states": [{ "time": 1.0 }] }]
        }"#;
        let doc: StoryboardDocument = serde_json::from_str(json).expect("parse");
        assert_eq!(doc.sprites.len(), 1);
        assert_eq!(doc.controllers.len(), 1);
    }

    #[test]
    fn parses_json_with_line_comments() {
        let json = r#"{
            // intro block
            "sprites": [{ "path": "a.png", "states": [{ "time": 0.0 }] }]
        }"#;
        let doc = StoryboardDocument::from_json_text(json).expect("parse");
        assert_eq!(doc.sprites.len(), 1);
    }
}
