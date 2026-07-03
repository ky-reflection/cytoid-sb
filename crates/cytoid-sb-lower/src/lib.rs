use cytoid_sb_model::StoryboardDocument;

/// Lower authoring sugar (templates, note selectors, time expressions) into plain JSON.
///
/// Phase 1: identity pass — sugar expansion lands in a follow-up PR.
pub fn lower_document(doc: StoryboardDocument) -> StoryboardDocument {
    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_lower_is_noop_for_now() {
        let doc = StoryboardDocument::default();
        assert_eq!(lower_document(doc.clone()), doc);
    }
}
