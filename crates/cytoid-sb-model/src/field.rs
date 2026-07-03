use serde_json::Value;

/// Read the first matching key from a JSON object (supports compiled PascalCase and lab snake_case).
pub fn object_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let obj = value.as_object()?;
    for key in keys {
        if let Some(found) = obj.get(*key) {
            return Some(found);
        }
    }
    None
}

pub fn object_field_str(value: &Value, keys: &[&str]) -> Option<String> {
    object_field(value, keys).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        _ => None,
    })
}

pub fn object_id(value: &Value) -> Option<String> {
    object_field_str(value, &["Id", "id"])
}

pub fn object_states(value: &Value) -> Vec<&Value> {
    object_field(value, &["States", "states"])
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default()
}

pub fn has_root_or_state_content(value: &Value) -> bool {
    !object_states(value).is_empty() || object_field(value, &["time", "Time"]).is_some()
}

pub fn has_id_target_conflict(value: &Value) -> bool {
    object_field(value, &["id", "Id"]).is_some()
        && object_field(value, &["target_id", "TargetId"]).is_some()
}

pub fn has_target_parent_conflict(value: &Value) -> bool {
    object_field(value, &["target_id", "TargetId"]).is_some()
        && object_field(value, &["parent_id", "ParentId"]).is_some()
}

pub fn template_ref(value: &Value) -> Option<String> {
    object_field_str(value, &["template", "Template"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_snake_and_pascal_keys() {
        let v = json!({"id": "a", "Path": "x.png"});
        assert_eq!(object_id(&v), Some("a".into()));
        assert_eq!(
            object_field_str(&v, &["Path", "path"]),
            Some("x.png".into())
        );
    }
}
