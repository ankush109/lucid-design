use anyhow::{Context, Result};
use regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::session::{Mode, Session};

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub slug: String,
    pub name: String,
    pub updated_at: u64,
    pub size: u64,
}

pub fn dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let d = PathBuf::from(home).join("Documents").join("lucid-design");
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
    let _ = fs::remove_file(d.join(format!("{}.skeleton.html", slug)));
    let _ = fs::remove_file(d.join(format!("{}.name", slug)));
    let _ = fs::remove_file(d.join(format!("{}.chat.json", slug)));
    let _ = fs::remove_file(d.join(format!("{}.chat.jsonl", slug)));
    let _ = fs::remove_file(d.join(format!("{}.session.json", slug)));
    let _ = fs::remove_file(d.join(format!("{}.pages.json", slug)));
    // Nuke any sub-pages ({slug}--*.html).
    if let Ok(entries) = fs::read_dir(&d) {
        let prefix = format!("{}--", slug);
        for e in entries.flatten() {
            if let Some(name) = e.file_name().to_str() {
                if name.starts_with(&prefix) && name.ends_with(".html") {
                    let _ = fs::remove_file(e.path());
                }
            }
        }
    }
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

// ══════════════════════════════════════════════════════════════════════════════
// Multi-page support
//
// Storage layout (backward compatible with single-file projects):
//   {slug}.html                ← "home" page (also the primary/first page)
//   {slug}--{page-slug}.html   ← additional pages (settings, users, ...)
//   {slug}.pages.json          ← manifest: [{slug,name}, ...] + active page
//   {slug}.name, {slug}.chat.json ← unchanged
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// "home" for the main page, otherwise a slug like "settings", "users".
    pub slug: String,
    /// Display name: "Home", "Settings", "Users".
    pub name: String,
    /// `false` when only the skeleton wireframe exists (auto-generated after
    /// home page). `true` once the user has upgraded the page to full
    /// fidelity via the "Build this page" action.
    #[serde(default = "default_true")]
    pub built: bool,
    /// `true` once the batch skeleton generator has produced a wireframe
    /// for this page (i.e. `{slug}--{page}.skeleton.html` exists).
    #[serde(default)]
    pub has_skeleton: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagesManifest {
    pub pages:  Vec<PageInfo>,
    pub active: String,
}

impl Default for PagesManifest {
    fn default() -> Self {
        Self {
            pages:  vec![PageInfo { slug: "home".into(), name: "Home".into(), built: true, has_skeleton: false }],
            active: "home".into(),
        }
    }
}

pub fn read_pages_manifest(project_slug: &str) -> Result<PagesManifest> {
    let d = dir()?;
    let path = d.join(format!("{}.pages.json", project_slug));
    if !path.exists() { return Ok(PagesManifest::default()); }
    let s = fs::read_to_string(&path).unwrap_or_default();
    if s.trim().is_empty() { return Ok(PagesManifest::default()); }
    Ok(serde_json::from_str(&s).unwrap_or_default())
}

pub fn write_pages_manifest(project_slug: &str, m: &PagesManifest) -> Result<()> {
    let d = dir()?;
    fs::write(
        d.join(format!("{}.pages.json", project_slug)),
        serde_json::to_string(m).unwrap_or_else(|_| "{}".into()),
    )?;
    Ok(())
}

fn page_file(project_slug: &str, page_slug: &str) -> PathBuf {
    let d = dir().unwrap_or_else(|_| PathBuf::from("."));
    if page_slug == "home" {
        d.join(format!("{}.html", project_slug))
    } else {
        d.join(format!("{}--{}.html", project_slug, page_slug))
    }
}

pub fn read_page(project_slug: &str, page_slug: &str) -> Result<String> {
    let path = page_file(project_slug, page_slug);
    Ok(fs::read_to_string(&path)?)
}

pub fn write_page(project_slug: &str, page_slug: &str, html: &str) -> Result<()> {
    let path = page_file(project_slug, page_slug);
    fs::write(&path, html)?;
    Ok(())
}

/// Skeleton-file counterpart. `{slug}--{page}.skeleton.html` for sub-pages;
/// `{slug}.skeleton.html` for the home page. Kept as a sibling so we can
/// toggle between the wireframe and the built version freely.
fn skeleton_file(project_slug: &str, page_slug: &str) -> PathBuf {
    let d = dir().unwrap_or_else(|_| PathBuf::from("."));
    if page_slug == "home" {
        d.join(format!("{}.skeleton.html", project_slug))
    } else {
        d.join(format!("{}--{}.skeleton.html", project_slug, page_slug))
    }
}

pub fn read_skeleton(project_slug: &str, page_slug: &str) -> Result<String> {
    let path = skeleton_file(project_slug, page_slug);
    Ok(fs::read_to_string(&path)?)
}

pub fn write_skeleton(project_slug: &str, page_slug: &str, html: &str) -> Result<()> {
    let path = skeleton_file(project_slug, page_slug);
    fs::write(&path, html)?;
    Ok(())
}

pub fn has_built_page(project_slug: &str, page_slug: &str) -> bool {
    page_file(project_slug, page_slug).exists()
        && !std::fs::read_to_string(page_file(project_slug, page_slug))
            .unwrap_or_default()
            .trim()
            .is_empty()
}

pub fn has_skeleton(project_slug: &str, page_slug: &str) -> bool {
    skeleton_file(project_slug, page_slug).exists()
}

/// Add a new sub-page to a project. Slugifies the display name and dedupes.
/// Returns the resolved page slug.
pub fn add_page(project_slug: &str, display_name: &str) -> Result<String> {
    let base = slugify(display_name);
    if base == "home" || base.is_empty() {
        return Err(anyhow::anyhow!("invalid page name: {}", display_name));
    }
    let mut manifest = read_pages_manifest(project_slug)?;

    // Dedupe: if a page with the exact slug exists, just make it active.
    if manifest.pages.iter().any(|p| p.slug == base) {
        manifest.active = base.clone();
        write_pages_manifest(project_slug, &manifest)?;
        return Ok(base);
    }

    let display = display_name.trim();
    let display = if display.is_empty() { &base } else { display };
    manifest.pages.push(PageInfo { slug: base.clone(), name: display.into(), built: true, has_skeleton: false });
    manifest.active = base.clone();
    write_pages_manifest(project_slug, &manifest)?;
    // Ensure an empty file exists so read_page doesn't fail before first generation.
    let path = page_file(project_slug, &base);
    if !path.exists() { fs::write(&path, "")?; }
    Ok(base)
}

pub fn set_active_page(project_slug: &str, page_slug: &str) -> Result<()> {
    let mut manifest = read_pages_manifest(project_slug)?;
    if !manifest.pages.iter().any(|p| p.slug == page_slug) {
        return Err(anyhow::anyhow!("page '{}' not in project '{}'", page_slug, project_slug));
    }
    manifest.active = page_slug.into();
    write_pages_manifest(project_slug, &manifest)
}

pub fn delete_page(project_slug: &str, page_slug: &str) -> Result<()> {
    if page_slug == "home" {
        return Err(anyhow::anyhow!("cannot delete the home page — delete the project instead"));
    }
    let d = dir()?;
    let _ = fs::remove_file(d.join(format!("{}--{}.html", project_slug, page_slug)));

    let mut manifest = read_pages_manifest(project_slug)?;
    manifest.pages.retain(|p| p.slug != page_slug);
    if manifest.active == page_slug { manifest.active = "home".into(); }
    write_pages_manifest(project_slug, &manifest)
}

// ══════════════════════════════════════════════════════════════════════════════
// Session persistence
//
// Storage:
//   {slug}.session.json  ← serialized Session (mode, brief, active_page, tokens…)
//   {slug}.chat.jsonl    ← structured chat log, one JSON message per line
// ══════════════════════════════════════════════════════════════════════════════

fn session_path(project_slug: &str) -> Result<PathBuf> {
    Ok(dir()?.join(format!("{}.session.json", project_slug)))
}

fn chat_jsonl_path(project_slug: &str) -> Result<PathBuf> {
    Ok(dir()?.join(format!("{}.chat.jsonl", project_slug)))
}

/// Read a project's persisted session. If missing, synthesize one from the
/// home HTML archetype so legacy projects self-upgrade with no user prompt,
/// and write it back so the classifier only runs once per project.
pub fn read_session(project_slug: &str) -> Result<Session> {
    let path = session_path(project_slug)?;
    if path.exists() {
        let s = fs::read_to_string(&path).unwrap_or_default();
        if let Ok(mut sess) = serde_json::from_str::<Session>(&s) {
            // Older session files may not carry a brief. If the home HTML
            // has a <title>, use it — beats falling back to the project's
            // display name (which is often terse like "Ok" or "Fintech")
            // and causes try_different_layout to design against the wrong
            // subject.
            if sess.brief.is_empty() {
                if let Some(t) = read(project_slug).ok().as_deref().and_then(extract_title) {
                    sess.brief = t;
                    let _ = write_session(project_slug, &sess);
                }
            }
            return Ok(sess);
        }
    }
    // Synthesize from the home HTML if present.
    let mut sess = Session::default();
    if let Ok(html) = read(project_slug) {
        sess.mode = classify_html_mode(&html);
        if let Some(t) = extract_title(&html) { sess.brief = t; }
    }
    let _ = write_session(project_slug, &sess);
    Ok(sess)
}

/// Pull the `<title>` text out of an HTML document, best-effort.
fn extract_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?is)<title[^>]*>([^<]{1,200})</title>").ok()?;
    let cap = re.captures(html)?;
    let s = cap.get(1)?.as_str().trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

pub fn write_session(project_slug: &str, sess: &Session) -> Result<()> {
    let path = session_path(project_slug)?;
    // Atomic write: tmp + rename so a crash mid-write can't corrupt the file.
    let tmp = path.with_extension("json.tmp");
    let body = serde_json::to_string_pretty(sess)
        .unwrap_or_else(|_| "{}".into());
    fs::write(&tmp, body)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Best-effort mode classifier from a rendered HTML string. `sidebar`,
/// `dashboard`, or `admin` in the archetype meta or id/class landscape → App;
/// otherwise → Landing (never Ambiguous — legacy projects always resolve
/// to something concrete).
fn classify_html_mode(html: &str) -> Mode {
    let lower = html.to_ascii_lowercase();
    let app_signals = [
        "name=\"archetype\" content=\"sidebar",
        "name=\"archetype\" content=\"dashboard",
        "name=\"archetype\" content=\"admin",
        "id=\"sidebar",
        "class=\"sidebar",
        "id=\"topbar",
        "id=\"dashboard",
        "class=\"dashboard",
    ];
    for sig in app_signals {
        if lower.contains(sig) { return Mode::App; }
    }
    Mode::Landing
}

/// Append a single chat message to the JSONL log. Each line is a self-
/// describing JSON object like `{"role":"user","kind":"text","content":"…"}`.
/// Structure matches whatever the React store sends via the `save_chat_line`
/// IPC — this file just persists the bytes verbatim.
pub fn append_chat_line(project_slug: &str, json_line: &str) -> Result<()> {
    let path = chat_jsonl_path(project_slug)?;
    let mut f = fs::OpenOptions::new().create(true).append(true).open(&path)?;
    // Enforce single-line-per-entry: strip embedded newlines from the payload.
    let one_line = json_line.replace('\n', " ").replace('\r', " ");
    writeln!(f, "{}", one_line.trim())?;
    Ok(())
}

/// Rewrite the entire chat log from a JSON array (used when the React store
/// mirrors its full message list — matches the legacy `save_chat` semantics
/// but with per-line JSON so streaming reads and appends are trivial).
pub fn overwrite_chat_from_array(project_slug: &str, json_array: &str) -> Result<()> {
    let path = chat_jsonl_path(project_slug)?;
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_array).unwrap_or_default();
    let tmp = path.with_extension("jsonl.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        for entry in arr {
            let line = serde_json::to_string(&entry).unwrap_or_default();
            writeln!(f, "{}", line)?;
        }
    }
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Read the chat log back as a JSON array string, one message per element.
/// Missing file → "[]". Corrupt lines are skipped rather than failing.
pub fn read_chat_jsonl(project_slug: &str) -> Result<String> {
    let path = chat_jsonl_path(project_slug)?;
    if !path.exists() { return Ok("[]".into()); }
    let s = fs::read_to_string(&path).unwrap_or_default();
    let mut out: Vec<serde_json::Value> = Vec::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            out.push(v);
        }
    }
    Ok(serde_json::to_string(&out).unwrap_or_else(|_| "[]".into()))
}
