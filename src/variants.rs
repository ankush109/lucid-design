// Variant library — hand-authored HTML/CSS blocks compiled into the binary.
// Parsed from `src/ai/variants.md`, `src/ai/palettes.md`, `src/ai/themes.md`
// on startup, exposed via a process-global `Library` singleton.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Variant {
    pub id:           String,
    pub display_name: String,
    pub category:     String,
    pub tags:         Vec<String>,
    pub placeholders: Vec<String>,
    pub style:        String,
    pub html:         String,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub id:   String,
    pub tags: Vec<String>,
    pub body: String,          // raw CSS custom property block (properties only)
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub id:   String,
    pub tags: Vec<String>,
    pub meta: HashMap<String, String>,   // palette, fonts, radius, spacing-base, section-gap, motion, density, signature, tone
}

pub struct Library {
    pub variants:    HashMap<String, Variant>,
    pub palettes:    HashMap<String, Palette>,
    pub themes:      HashMap<String, Theme>,
    pub by_category: HashMap<String, Vec<String>>,
}

impl Library {
    pub fn variants_in(&self, category: &str) -> Vec<&Variant> {
        self.by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.variants.get(id)).collect())
            .unwrap_or_default()
    }
}

pub fn library() -> &'static Library {
    static LIB: OnceLock<Library> = OnceLock::new();
    LIB.get_or_init(load)
}

fn load() -> Library {
    let variants = parse_variants(include_str!("ai/variants.md"));
    let palettes = parse_palettes(include_str!("ai/palettes.md"));
    let themes   = parse_themes(include_str!("ai/themes.md"));

    let mut by_category: HashMap<String, Vec<String>> = HashMap::new();
    for (id, v) in &variants {
        by_category.entry(v.category.clone()).or_default().push(id.clone());
    }
    for ids in by_category.values_mut() { ids.sort(); }

    Library { variants, palettes, themes, by_category }
}

fn parse_variants(md: &str) -> HashMap<String, Variant> {
    let mut out = HashMap::new();
    // Each variant block is preceded by "## " at line start. Split on "\n## ".
    for chunk in md.split("\n## ").skip(1) {
        let mut lines = chunk.lines();
        let header = lines.next().unwrap_or("");
        let (id_part, display) = match header.split_once(':') {
            Some((a, b)) => (a.trim().to_string(), b.trim().to_string()),
            None         => (header.trim().to_string(), header.trim().to_string()),
        };

        let mut category = String::new();
        let mut tags: Vec<String> = Vec::new();
        let mut placeholders: Vec<String> = Vec::new();
        let mut style = String::new();
        let mut html  = String::new();
        let mut mode  = "meta";

        for line in lines {
            match mode {
                "meta" => {
                    let trimmed = line.trim();
                    if trimmed == "---STYLE---" { mode = "style"; }
                    else if trimmed == "---HTML---" { mode = "html"; }
                    else if line.starts_with("[category:") {
                        category = extract_bracket(line, "category");
                    } else if line.starts_with("[tags:") {
                        tags = extract_bracket(line, "tags")
                            .split(',').map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()).collect();
                    } else if line.starts_with("[placeholders:") {
                        placeholders = extract_bracket(line, "placeholders")
                            .split(',').map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()).collect();
                    }
                }
                "style" => {
                    if line.trim() == "---HTML---" { mode = "html"; continue; }
                    style.push_str(line); style.push('\n');
                }
                "html" => {
                    if line.trim() == "---" { break; }
                    html.push_str(line); html.push('\n');
                }
                _ => {}
            }
        }

        if id_part.is_empty() || category.is_empty() || html.trim().is_empty() { continue; }
        out.insert(id_part.clone(), Variant {
            id: id_part.clone(),
            display_name: display,
            category, tags, placeholders,
            style: style.trim().to_string(),
            html:  html.trim().to_string(),
        });
    }
    out
}

fn parse_palettes(md: &str) -> HashMap<String, Palette> {
    let mut out = HashMap::new();
    for chunk in md.split("\n## ").skip(1) {
        let mut lines = chunk.lines();
        let id = lines.next().unwrap_or("").trim().to_string();
        if id.is_empty() { continue; }

        let mut tags: Vec<String> = Vec::new();
        let mut body = String::new();
        let mut in_body = false;
        for line in lines {
            if line.starts_with("[tags:") {
                tags = extract_bracket(line, "tags")
                    .split(',').map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()).collect();
            } else if line.trim() == "---" {
                in_body = true; continue;
            } else if in_body {
                if line.starts_with("## ") { break; }
                if !line.trim().is_empty() { body.push_str(line); body.push('\n'); }
            }
        }
        if body.trim().is_empty() { continue; }
        out.insert(id.clone(), Palette { id, tags, body: body.trim().to_string() });
    }
    out
}

fn parse_themes(md: &str) -> HashMap<String, Theme> {
    let mut out = HashMap::new();
    for chunk in md.split("\n## ").skip(1) {
        let mut lines = chunk.lines();
        let id = lines.next().unwrap_or("").trim().to_string();
        if id.is_empty() { continue; }

        let mut tags: Vec<String> = Vec::new();
        let mut meta: HashMap<String, String> = HashMap::new();

        for line in lines {
            if line.starts_with("## ") { break; }
            let trimmed = line.trim();
            if !trimmed.starts_with('[') || !trimmed.ends_with(']') { continue; }
            let inner = &trimmed[1..trimmed.len()-1];
            let (key, value) = match inner.split_once(':') {
                Some((k, v)) => (k.trim().to_string(), v.trim().to_string()),
                None => continue,
            };
            if key == "tags" {
                tags = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            } else {
                meta.insert(key, value);
            }
        }
        out.insert(id.clone(), Theme { id, tags, meta });
    }
    out
}

fn extract_bracket(line: &str, key: &str) -> String {
    let needle = format!("[{}:", key);
    if let Some(pos) = line.find(&needle) {
        let after = &line[pos + needle.len()..];
        if let Some(end) = after.find(']') {
            return after[..end].trim().to_string();
        }
    }
    String::new()
}
