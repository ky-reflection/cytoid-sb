use crate::StoryboardDocument;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentSummary {
    pub texts: usize,
    pub sprites: usize,
    pub videos: usize,
    pub lines: usize,
    pub controllers: usize,
    pub note_controllers: usize,
    pub triggers: usize,
    pub templates: usize,
    pub compiled: bool,
}

impl DocumentSummary {
    pub fn object_count(&self) -> usize {
        self.texts
            + self.sprites
            + self.videos
            + self.lines
            + self.controllers
            + self.note_controllers
    }

    pub fn format_line(&self) -> String {
        let mut parts = Vec::new();
        if self.sprites > 0 {
            parts.push(format!("{} sprites", self.sprites));
        }
        if self.controllers > 0 {
            parts.push(format!("{} controllers", self.controllers));
        }
        if self.texts > 0 {
            parts.push(format!("{} texts", self.texts));
        }
        if self.videos > 0 {
            parts.push(format!("{} videos", self.videos));
        }
        if self.lines > 0 {
            parts.push(format!("{} lines", self.lines));
        }
        if self.note_controllers > 0 {
            parts.push(format!("{} note_controllers", self.note_controllers));
        }
        if self.triggers > 0 {
            parts.push(format!("{} triggers", self.triggers));
        }
        if self.templates > 0 {
            parts.push(format!("{} templates", self.templates));
        }
        if parts.is_empty() {
            "0 objects".into()
        } else {
            parts.join(", ")
        }
    }
}

impl From<&StoryboardDocument> for DocumentSummary {
    fn from(doc: &StoryboardDocument) -> Self {
        Self {
            texts: doc.texts.len(),
            sprites: doc.sprites.len(),
            videos: doc.videos.len(),
            lines: doc.lines.len(),
            controllers: doc.controllers.len(),
            note_controllers: doc.note_controllers.len(),
            triggers: doc.triggers.len(),
            templates: doc.templates.as_ref().map(|m| m.len()).unwrap_or(0),
            compiled: doc.compiled,
        }
    }
}
