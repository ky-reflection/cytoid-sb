use camino::{Utf8Path, Utf8PathBuf};
use cytoid_sb_diag::{SbError, SbResult};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    StoryboardJson,
    LuaScript,
    LevelDirectory,
}

pub fn detect_input(path: &Utf8Path) -> SbResult<InputKind> {
    let meta = fs::metadata(path.as_std_path()).map_err(|source| SbError::Io {
        path: path.to_string(),
        source,
    })?;

    if meta.is_dir() {
        return Ok(InputKind::LevelDirectory);
    }

    match path.extension() {
        Some("json") => Ok(InputKind::StoryboardJson),
        Some("lua") => Ok(InputKind::LuaScript),
        Some(ext) => Err(SbError::UnsupportedInput { kind: ext.into() }),
        None => Err(SbError::UnsupportedInput {
            kind: "<no extension>".into(),
        }),
    }
}

pub fn resolve_storyboard_input(path: &Utf8Path, kind: InputKind) -> SbResult<Utf8PathBuf> {
    match kind {
        InputKind::StoryboardJson | InputKind::LuaScript => Ok(path.to_path_buf()),
        InputKind::LevelDirectory => find_storyboard_in_level_dir(path),
    }
}

fn find_storyboard_in_level_dir(path: &Utf8Path) -> SbResult<Utf8PathBuf> {
    for candidate in [
        "storyboard.lua",
        "storyboard.json",
        "Storyboard.json",
        "storyboard.generated.json",
        "Kami_storyboard.json",
    ] {
        let p = path.join(candidate);
        if p.exists() {
            return Ok(p);
        }
    }

    let mut json_candidates = Vec::new();
    let entries = fs::read_dir(path.as_std_path()).map_err(|source| SbError::Io {
        path: path.to_string(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| SbError::Io {
            path: path.to_string(),
            source,
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".json") {
            continue;
        }
        if lower.contains("generated") {
            continue;
        }
        if !lower.contains("storyboard") {
            continue;
        }
        let candidate = path.join(name.as_ref());
        let size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
        json_candidates.push((candidate, size));
    }

    json_candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
    if let Some((best, _)) = json_candidates.into_iter().next() {
        return Ok(best);
    }

    Err(SbError::Other {
        message: format!(
            "no storyboard.lua/json found in {}; expected storyboard.lua, storyboard.json, or *storyboard*.json",
            path
        ),
    })
}

/// Default output path for `compile` when `--output` is omitted.
pub fn default_generated_path(input: &Utf8Path) -> Utf8PathBuf {
    let base = if fs::metadata(input.as_std_path())
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
    {
        input
    } else {
        input.parent().unwrap_or_else(|| Utf8Path::new("."))
    };
    base.join("storyboard.generated.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prefers_storyboard_lua_in_level_dir() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("storyboard.json"), "{}").unwrap();
        fs::write(dir.path().join("storyboard.lua"), "-- lua").unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let resolved = resolve_storyboard_input(&path, InputKind::LevelDirectory).unwrap();
        assert_eq!(resolved.file_name(), Some("storyboard.lua"));
    }

    #[test]
    fn default_generated_path_for_lua_file() {
        let path = Utf8PathBuf::from("examples/hello/storyboard.lua");
        let out = default_generated_path(&path);
        assert_eq!(out.file_name(), Some("storyboard.generated.json"));
        assert_eq!(out.parent().and_then(|p| p.file_name()), Some("hello"));
    }

    #[test]
    fn default_generated_path_for_level_dir() {
        let dir = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let out = default_generated_path(&path);
        assert_eq!(out, path.join("storyboard.generated.json"));
    }

    #[test]
    fn finds_chaos_storyboard_json() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("level.json"), "{}").unwrap();
        fs::write(
            dir.path().join("CHAOS_storyboard.json"),
            r#"{"controllers":[]}"#,
        )
        .unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let resolved = resolve_storyboard_input(&path, InputKind::LevelDirectory).unwrap();
        assert_eq!(resolved.file_name(), Some("CHAOS_storyboard.json"));
    }
}
