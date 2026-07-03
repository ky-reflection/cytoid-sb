use camino::Utf8Path;
use cytoid_sb_diag::{SbError, SbResult};
use cytoid_sb_model::StoryboardDocument;
use std::fs;

pub struct EmitOptions {
    pub pretty: bool,
    pub mark_generated: bool,
}

impl Default for EmitOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            mark_generated: true,
        }
    }
}

pub fn emit_document(doc: &StoryboardDocument, options: &EmitOptions) -> SbResult<String> {
    let mut out = doc.clone();
    if options.mark_generated && !out.compiled {
        // Preserve author `compiled` when already set; otherwise mark as tool output.
        out.compiled = true;
    }

    let value = if out.compiled {
        compiled_document_value(&out)?
    } else {
        serde_json::to_value(&out).map_err(|e| SbError::Other {
            message: format!("failed to serialize storyboard: {e}"),
        })?
    };

    let json = if options.pretty {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    }
    .map_err(|e| SbError::Other {
        message: format!("failed to serialize storyboard: {e}"),
    })?;

    Ok(json)
}

fn compiled_document_value(doc: &StoryboardDocument) -> SbResult<serde_json::Value> {
    let mut value = serde_json::to_value(doc).map_err(|e| SbError::Other {
        message: format!("failed to serialize storyboard: {e}"),
    })?;
    let obj = value.as_object_mut().ok_or_else(|| SbError::Other {
        message: "serialized storyboard root is not an object".into(),
    })?;

    // Unity's compiled parse path casts every root collection to JArray without
    // null checks, matching Storyboard.Compile(). Keep empty arrays explicit.
    for key in [
        "texts",
        "sprites",
        "videos",
        "lines",
        "controllers",
        "note_controllers",
    ] {
        obj.entry(key)
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    }
    obj.insert("compiled".into(), serde_json::Value::Bool(true));
    Ok(value)
}

pub fn write_document(
    doc: &StoryboardDocument,
    path: &Utf8Path,
    options: &EmitOptions,
) -> SbResult<()> {
    let json = emit_document(doc, options)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SbError::Io {
            path: parent.to_string(),
            source,
        })?;
    }
    fs::write(path.as_std_path(), json).map_err(|source| SbError::Io {
        path: path.to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cytoid_sb_model::StoryboardDocument;
    use serde_json::json;

    #[test]
    fn marks_output_as_compiled_by_default() {
        let doc = StoryboardDocument {
            sprites: vec![json!({
                "Id": "g",
                "Path": "g.png",
                "States": [{"Time": 0.0}]
            })],
            ..Default::default()
        };
        let json = emit_document(&doc, &EmitOptions::default()).unwrap();
        assert!(json.contains("\"compiled\": true"));
        assert!(json.contains("\"texts\": []"));
        assert!(json.contains("\"controllers\": []"));
    }
}
