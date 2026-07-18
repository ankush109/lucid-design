use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBase {
    pub entries: Vec<DesignEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignEntry {
    pub idea: String,
    pub tone: String,
    pub colors: Vec<String>,
    pub fonts: Vec<String>,
    pub timestamp: u64,
}

impl KnowledgeBase {
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn add(&mut self, entry: DesignEntry) {
        self.entries.push(entry);
        if self.entries.len() > 30 {
            self.entries.remove(0);
        }
        let path = Self::path();
        if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
        if let Ok(s) = serde_json::to_string_pretty(self) { let _ = std::fs::write(path, s); }
    }

    /// Format recent entries as extra context for AI prompts.
    pub fn prompt_context(&self) -> String {
        if self.entries.is_empty() { return String::new(); }
        let mut out = String::from("\n\n## Proven design patterns from past sessions (use as inspiration):\n");
        for e in self.entries.iter().rev().take(6) {
            out.push_str(&format!(
                "- idea='{}' tone='{}' → colors: {} | fonts: {}\n",
                e.idea, e.tone,
                if e.colors.is_empty() { "—".into() } else { e.colors.join(", ") },
                if e.fonts.is_empty()  { "—".into() } else { e.fonts.join(", ") },
            ));
        }
        out
    }

    fn path() -> PathBuf {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
            .join(".config").join("design-gen").join("knowledge.json")
    }
}
