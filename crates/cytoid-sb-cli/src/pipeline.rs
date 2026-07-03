use crate::input::{detect_input, resolve_storyboard_input, InputKind};
use camino::Utf8Path;
use cytoid_sb_diag::{SbError, SbResult};
use cytoid_sb_emit::{write_document, EmitOptions};
use cytoid_sb_lower::lower_document;
use cytoid_sb_model::{DocumentSummary, StoryboardDocument};
use cytoid_sb_validate::{validate_document, ValidationReport};
use std::fs;

pub struct PipelineReport {
    pub summary: DocumentSummary,
    pub validation: ValidationReport,
}

impl PipelineReport {
    pub fn is_ok(&self) -> bool {
        self.validation.is_ok()
    }

    pub fn object_count(&self) -> usize {
        self.summary.object_count()
    }
}

pub fn load_document(path: &Utf8Path) -> SbResult<StoryboardDocument> {
    let text = fs::read_to_string(path.as_std_path()).map_err(|source| SbError::Io {
        path: path.to_string(),
        source,
    })?;
    StoryboardDocument::from_json_text(&text).map_err(|source| SbError::JsonParse {
        path: path.to_string(),
        source,
    })
}

fn load_or_compile(path: &Utf8Path, kind: InputKind) -> SbResult<StoryboardDocument> {
    match kind {
        InputKind::StoryboardJson => {
            let json_path = resolve_storyboard_input(path, kind)?;
            load_document(&json_path)
        }
        InputKind::LuaScript => cytoid_sb_lua::compile_file(path),
        InputKind::LevelDirectory => {
            let input_path = resolve_storyboard_input(path, kind)?;
            let nested_kind = detect_input(&input_path)?;
            load_or_compile(&input_path, nested_kind)
        }
    }
}

fn finalize(doc: StoryboardDocument) -> (StoryboardDocument, ValidationReport) {
    let lowered = lower_document(doc);
    let validation = validate_document(&lowered);
    (lowered, validation)
}

pub fn check_path(path: &Utf8Path) -> SbResult<PipelineReport> {
    let kind = detect_input(path)?;
    let doc = load_or_compile(path, kind)?;
    let (lowered, validation) = finalize(doc);
    Ok(PipelineReport {
        summary: lowered.summary(),
        validation,
    })
}

pub fn compile_path(input: &Utf8Path, output: &Utf8Path) -> SbResult<PipelineReport> {
    let kind = detect_input(input)?;
    let doc = load_or_compile(input, kind)?;
    let (lowered, validation) = finalize(doc);

    let report = PipelineReport {
        summary: lowered.summary(),
        validation: validation.clone(),
    };

    if !validation.is_ok() {
        return Err(SbError::Validation {
            count: validation.issues.len(),
        });
    }

    write_document(
        &lowered,
        output,
        &EmitOptions {
            pretty: true,
            mark_generated: should_mark_compiled(&lowered),
        },
    )?;

    Ok(report)
}

fn should_mark_compiled(doc: &StoryboardDocument) -> bool {
    doc.compiled
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use cytoid_sb_model::StoryboardDocument;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn check_example_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("storyboard.json");
        let doc = StoryboardDocument {
            sprites: vec![json!({
                "Id": "g",
                "Path": "g.png",
                "States": [{"Time": 0.0, "Opacity": 1.0}]
            })],
            ..Default::default()
        };
        std::fs::write(&path, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
        let utf8 = Utf8PathBuf::from_path_buf(path).unwrap();
        let report = check_path(&utf8).unwrap();
        assert!(report.is_ok());
    }

    #[test]
    fn compiles_lua_example() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/hello/storyboard.lua");
        let out = path.parent().unwrap().join("storyboard.generated.json");
        let report = compile_path(&path, &out).unwrap();
        assert!(report.is_ok());
        assert_eq!(report.object_count(), 1);
        assert_eq!(report.summary.sprites, 1);

        let json = std::fs::read_to_string(out.as_std_path()).unwrap();
        assert!(!json.contains("\"compiled\": true"));
        assert!(json.contains("\"path\""));
    }

    #[test]
    fn keeps_uncompiled_json_uncompiled() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("storyboard.json");
        let output = dir.path().join("storyboard.generated.json");
        std::fs::write(
            &input,
            r#"{"sprites":[{"path":"a.png","states":[{"time":0,"opacity":1}]}]}"#,
        )
        .unwrap();
        let input = Utf8PathBuf::from_path_buf(input).unwrap();
        let output = Utf8PathBuf::from_path_buf(output).unwrap();
        compile_path(&input, &output).unwrap();
        let json = std::fs::read_to_string(output.as_std_path()).unwrap();
        assert!(!json.contains("\"compiled\": true"));
        assert!(json.contains("\"path\""));
    }

    #[test]
    fn checks_hello_json() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/hello/storyboard.json");
        let report = check_path(&path).unwrap();
        assert!(report.is_ok());
        assert_eq!(report.summary.sprites, 1);
    }

    #[test]
    fn checks_kou_ppap_lua_example() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/kou_ppap/storyboard.lua");
        if !path.exists() {
            return;
        }
        let report = check_path(&path).unwrap();
        assert!(report.is_ok());
        assert_eq!(report.summary.sprites, 1);
        assert_eq!(report.summary.controllers, 2);
    }

    #[test]
    fn checks_kyr_chaos_lua_example() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/kyr_bite_your_nails/storyboard.lua");
        if !path.exists() {
            return;
        }
        let report = check_path(&path).unwrap();
        assert!(report.is_ok());
        assert_eq!(report.summary.note_controllers, 26);
        assert_eq!(report.summary.controllers, 4);
    }

    #[test]
    fn checks_gate_showcase_lua_example() {
        let path = Utf8PathBuf::from_path_buf(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .join("../../examples/gate_showcase/storyboard.lua");
        let report = check_path(&path).unwrap();
        assert!(report.is_ok());
        assert_eq!(report.summary.sprites, 6);
        assert_eq!(report.summary.texts, 1);
        assert_eq!(report.summary.videos, 1);
        assert_eq!(report.summary.lines, 1);
        assert_eq!(report.summary.note_controllers, 12);
        assert_eq!(report.summary.triggers, 1);
    }
}
