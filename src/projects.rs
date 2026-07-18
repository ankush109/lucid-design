use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub slug: String,
    pub name: String,
    pub updated_at: u64,
    pub size: u64,
}

pub fn dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let d = PathBuf::from(home).join("Documents").join("design-gen");
    if !d.exists() {
        fs::create_dir_all(&d).context("create projects dir")?;
    }
    Ok(d)
}

pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') { out.pop(); }
    if out.is_empty() { out.push_str("untitled"); }
    out
}

fn unslugify(slug: &str) -> String {
    slug.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn list() -> Result<Vec<Project>> {
    let d = dir()?;
    let mut items: Vec<Project> = Vec::new();
    for entry in fs::read_dir(&d)? {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("html") { continue; }
        let slug = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(), None => continue
        };
        let name = name_of(&slug).unwrap_or_else(|_| unslugify(&slug));
        let md = match entry.metadata() { Ok(m) => m, Err(_) => continue };
        let updated_at = md.modified().ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()).unwrap_or(0);
        items.push(Project { slug, name, updated_at, size: md.len() });
    }
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(items)
}

pub fn name_of(slug: &str) -> Result<String> {
    let d = dir()?;
    let name_path = d.join(format!("{}.name", slug));
    let name = fs::read_to_string(&name_path).ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| unslugify(slug));
    Ok(name)
}

fn resolve_slug(name: &str) -> Result<String> {
    let d = dir()?;
    let base = slugify(name);
    let mut slug = base.clone();
    let mut i = 2;
    while d.join(format!("{}.html", slug)).exists() {
        slug = format!("{}-{}", base, i);
        i += 1;
    }
    Ok(slug)
}

pub fn create(name: &str) -> Result<Project> {
    let display = name.trim();
    let display = if display.is_empty() { "Untitled" } else { display };
    let slug = resolve_slug(display)?;
    let d = dir()?;
    fs::write(d.join(format!("{}.html", slug)), "")?;
    fs::write(d.join(format!("{}.name", slug)), display)?;
    let md = fs::metadata(d.join(format!("{}.html", slug)))?;
    let updated_at = md.modified().ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs()).unwrap_or(0);
    Ok(Project { slug, name: display.to_string(), updated_at, size: md.len() })
}

pub fn read(slug: &str) -> Result<String> {
    let d = dir()?;
    Ok(fs::read_to_string(d.join(format!("{}.html", slug)))?)
}

pub fn write(slug: &str, html: &str) -> Result<()> {
    let d = dir()?;
    fs::write(d.join(format!("{}.html", slug)), html)?;
    Ok(())
}

pub fn delete(slug: &str) -> Result<()> {
    let d = dir()?;
    let _ = fs::remove_file(d.join(format!("{}.html", slug)));
    let _ = fs::remove_file(d.join(format!("{}.name", slug)));
    let _ = fs::remove_file(d.join(format!("{}.chat.json", slug)));
    Ok(())
}

/// Read the chat log JSON for a project. Returns "[]" if missing or empty.
pub fn read_chat(slug: &str) -> Result<String> {
    let d = dir()?;
    let path = d.join(format!("{}.chat.json", slug));
    if !path.exists() { return Ok("[]".into()); }
    let s = fs::read_to_string(&path).unwrap_or_else(|_| "[]".into());
    if s.trim().is_empty() { Ok("[]".into()) } else { Ok(s) }
}

pub fn write_chat(slug: &str, json: &str) -> Result<()> {
    let d = dir()?;
    fs::write(d.join(format!("{}.chat.json", slug)), json)?;
    Ok(())
}
