use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct DesignRef {
    pub name: String,
    pub tags: Vec<String>,
    pub site_url: String,
    pub colors: Vec<String>,
    pub fonts: Vec<String>,
}

impl DesignRef {
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("- {}", self.name)];
        if !self.tags.is_empty()   { parts.push(format!("  tags: {}", self.tags.join(", "))); }
        if !self.colors.is_empty() { parts.push(format!("  colors: {}", self.colors.join(", "))); }
        if !self.fonts.is_empty()  { parts.push(format!("  fonts: {}", self.fonts.join(", "))); }
        parts.join("\n")
    }
}

fn build_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .unwrap()
}

pub async fn gather(idea: &str) -> String {
    let client = build_client();
    let keyword = idea.split_whitespace().take(3).collect::<Vec<_>>().join("+");

    let (aww, si) = tokio::join!(
        scrape_awwwards(&client, &keyword),
        scrape_siteinspire(&client, &keyword),
    );

    let mut refs: Vec<DesignRef> = Vec::new();
    if let Ok(mut r) = aww { refs.append(&mut r); }
    if let Ok(mut r) = si  { refs.append(&mut r); }

    let enriched = enrich_with_tokens(&client, refs).await;

    if enriched.is_empty() {
        format!("No live references scraped for '{}'. Use your design knowledge.", keyword)
    } else {
        enriched.iter().map(|r| r.summary()).collect::<Vec<_>>().join("\n\n")
    }
}

async fn scrape_awwwards(client: &Client, keyword: &str) -> Result<Vec<DesignRef>> {
    let url = format!("https://www.awwwards.com/websites/?search={}", keyword);
    let html = client.get(&url).send().await?.text().await?;

    // Parse synchronously — no awaits while holding ElementRef
    let refs = {
        let doc = Html::parse_document(&html);
        let card_sel = Selector::parse("article.js-item").unwrap();
        let name_sel = Selector::parse("h3, h2, .title").unwrap();
        let tag_sel  = Selector::parse(".categories a, .tags a").unwrap();
        let link_sel = Selector::parse("a[href]").unwrap();

        doc.select(&card_sel).take(3).filter_map(|card| {
            let name = card.select(&name_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())?;
            if name.is_empty() { return None; }

            let tags = card.select(&tag_sel)
                .map(|e| e.text().collect::<String>().trim().to_string())
                .filter(|t| !t.is_empty()).collect();

            let site_url = card.select(&link_sel)
                .filter_map(|e| e.value().attr("href"))
                .find(|h| h.starts_with("http") && !h.contains("awwwards"))
                .unwrap_or("").to_string();

            Some(DesignRef { name, tags, site_url, colors: vec![], fonts: vec![] })
        }).collect::<Vec<_>>()
    };
    Ok(refs)
}

async fn scrape_siteinspire(client: &Client, keyword: &str) -> Result<Vec<DesignRef>> {
    let url = format!("https://www.siteinspire.com/?search={}", keyword);
    let html = client.get(&url).send().await?.text().await?;

    let refs = {
        let doc = Html::parse_document(&html);
        let card_sel = Selector::parse(".site").unwrap();
        let name_sel = Selector::parse(".title, h3, h2").unwrap();
        let tag_sel  = Selector::parse(".style, .type, .category").unwrap();
        let link_sel = Selector::parse("a[href]").unwrap();

        doc.select(&card_sel).take(3).filter_map(|card| {
            let name = card.select(&name_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())?;
            if name.is_empty() { return None; }

            let tags = card.select(&tag_sel)
                .map(|e| e.text().collect::<String>().trim().to_string())
                .filter(|t| !t.is_empty()).collect();

            let site_url = card.select(&link_sel)
                .filter_map(|e| e.value().attr("href"))
                .find(|h| h.starts_with("http") && !h.contains("siteinspire"))
                .unwrap_or("").to_string();

            Some(DesignRef { name, tags, site_url, colors: vec![], fonts: vec![] })
        }).collect::<Vec<_>>()
    };
    Ok(refs)
}

async fn enrich_with_tokens(client: &Client, mut refs: Vec<DesignRef>) -> Vec<DesignRef> {
    for r in refs.iter_mut().take(3) {
        if r.site_url.is_empty() { continue; }
        if let Ok((colors, fonts)) = extract_css_tokens(client, &r.site_url).await {
            r.colors = colors;
            r.fonts  = fonts;
        }
    }
    refs
}

/// Fetch a single user-provided URL and extract a structured design reference:
/// palette, fonts, headline copy, and actual image URLs. Used when the user
/// pastes a URL in the idea so the LLM can rebuild it better while reusing
/// the site's real imagery.
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub url:  String,
    pub alt:  String,
    pub w:    Option<u32>,
    pub h:    Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ReferenceSite {
    pub url:          String,
    pub title:        String,
    pub description:  String,
    pub colors:       Vec<String>,
    pub fonts:        Vec<String>,
    pub headings:     Vec<String>,
    pub body_snippet: String,
    pub images:       Vec<ImageAsset>,

    // Visual-sophistication tokens — these are what make the reference feel
    // polished. Extract them so the LLM can match motion, depth, and rounding
    // even without vision.
    pub animations:   Vec<String>,
    pub transitions:  Vec<String>,
    pub gradients:    Vec<String>,
    pub shadows:      Vec<String>,
    pub radii:        Vec<String>,
    pub keyframes:    Vec<String>,

    // Path to a saved screenshot of the reference (via headless Chrome), if one
    // was successfully captured. Currently informational — only Anthropic direct
    // and OpenAI GPT-4V would consume it in a follow-up integration.
    pub screenshot:   Option<std::path::PathBuf>,

    // 3D / WebGL detection — if true, LLM is instructed to include a matching
    // three.js signature element in the new design.
    pub has_threejs:  bool,
    pub has_webgl:    bool,

    // Full CSS keyframes blocks and inline SVG markup — these are what carry
    // the SPECIFIC visual richness (rocket illustrations, orbital paths, etc.)
    // that generic token extraction misses.
    pub keyframe_blocks: Vec<String>,
    pub inline_svgs:     Vec<String>,
}

impl ReferenceSite {
    pub fn as_prompt_block(&self) -> String {
        let mut out = String::from("=== USER-PROVIDED REFERENCE SITE ===\n");
        out.push_str(&format!("URL: {}\n", self.url));
        if !self.title.is_empty()       { out.push_str(&format!("Title: {}\n", self.title)); }
        if !self.description.is_empty() { out.push_str(&format!("Meta description: {}\n", self.description)); }
        if !self.colors.is_empty()      { out.push_str(&format!("Detected palette (hex): {}\n", self.colors.join(", "))); }
        if !self.fonts.is_empty()       { out.push_str(&format!("Detected fonts: {}\n", self.fonts.join(", "))); }
        if !self.headings.is_empty() {
            out.push_str("Sample headings:\n");
            for h in &self.headings { out.push_str(&format!("  - {}\n", h)); }
        }
        if !self.body_snippet.is_empty() {
            out.push_str(&format!("Sample body copy: {}\n", self.body_snippet));
        }
        if !self.images.is_empty() {
            out.push_str("\nExtracted image URLs — USE THESE VERBATIM in the new design instead of picking from the IMAGE TOOLKIT. Listed in DOM order; earlier images are typically hero / above-the-fold.\n");
            for (i, img) in self.images.iter().enumerate() {
                let dims = match (img.w, img.h) {
                    (Some(w), Some(h)) => format!(" [{}x{}]", w, h),
                    _ => String::new(),
                };
                let alt = if img.alt.is_empty() { String::new() } else { format!(" (alt: {})", img.alt) };
                out.push_str(&format!("  {:>2}. {}{}{}\n", i + 1, img.url, dims, alt));
            }
        }
            // 3D signal — reference uses three.js / WebGL.
        if self.has_threejs || self.has_webgl {
            out.push_str("\n**REFERENCE USES 3D / WebGL** (");
            if self.has_threejs { out.push_str("three.js detected"); }
            else                { out.push_str("canvas + WebGL detected"); }
            out.push_str("). You MUST include a matching 3D signature element in the new design — see the THREE.JS TOOLKIT for patterns. Pick ONE (floating orb, particle drift, wireframe object, or gradient plane) that fits the subject.\n");
        }

    // Motion / depth tokens — the polish signals.
        if !self.animations.is_empty() {
            out.push_str("\nReference animations (matches these for motion polish):\n");
            for a in &self.animations { out.push_str(&format!("  · {}\n", a)); }
        }
        if !self.transitions.is_empty() {
            out.push_str("\nReference transitions (use similar durations & easings on interactive elements):\n");
            for t in &self.transitions.iter().take(8).collect::<Vec<_>>() { out.push_str(&format!("  · {}\n", t)); }
        }
        if !self.keyframes.is_empty() {
            out.push_str(&format!("\nReference declares @keyframes: {}\n", self.keyframes.join(", ")));
            out.push_str("You must define matching or equivalent @keyframes and USE them (fade-in on load, subtle hover lift, etc.). This is what makes the site feel alive.\n");
        }
        if !self.gradients.is_empty() {
            out.push_str("\nReference gradients (replicate similar treatments where the design calls for depth or luminosity):\n");
            for g in &self.gradients { out.push_str(&format!("  · {}\n", g)); }
        }
        if !self.shadows.is_empty() {
            out.push_str("\nReference shadows (match tone, blur, offset — DO NOT use pure black):\n");
            for s in &self.shadows { out.push_str(&format!("  · {}\n", s)); }
        }
        if !self.radii.is_empty() {
            out.push_str(&format!("\nReference border-radius values: {}. Pick 2-3 from this range and hold them consistently.\n", self.radii.join(", ")));
        }
        if let Some(ref path) = self.screenshot {
            out.push_str(&format!("\nReference screenshot saved locally at: {} (vision-model providers can consume this to match visual composition).\n", path.display()));
        }

        // Full keyframes bodies — this is where the actual motion lives (a rocket
        // arc, orbital rotation, particle drift path). Text-only LLMs need this
        // to reproduce specific motion, not just "some animation exists".
        if !self.keyframe_blocks.is_empty() {
            out.push_str("\nReference @keyframes bodies (rebuild these motions verbatim or as close as possible in the new design's own @keyframes):\n```css\n");
            for block in &self.keyframe_blocks {
                out.push_str(block);
                out.push_str("\n\n");
            }
            out.push_str("```\n");
        }

        // Inline SVGs — usually where illustrations (rockets, orbits, characters)
        // live. Redraw these in the new design, adapting colours to your palette.
        if !self.inline_svgs.is_empty() {
            out.push_str("\nReference inline SVG markup (these are the actual illustrations on the site — reuse them literally, or redraw matching silhouettes with the new palette. If a rocket / astronaut / orbital element appears, it MUST appear in the new design too):\n```svg\n");
            for svg in &self.inline_svgs {
                out.push_str(svg);
                out.push_str("\n\n");
            }
            out.push_str("```\n");
        }

        out.push_str("\nInstruction: Rebuild the reference for the user's new subject at a HIGHER quality tier. Preserve:\n  • The reference's palette (use the detected hex values as your color tokens).\n  • The reference's typography (use the detected font families).\n  • The reference's actual image URLs — embed them directly in the new design instead of inventing URLs from the IMAGE TOOLKIT.\n  • The reference's motion vocabulary — use animation/transition durations and easings similar to the ones listed. Add scroll-triggered fade-ins, hover-lift transforms, gradient shifts. This is what makes designs feel alive rather than static.\n  • The reference's depth vocabulary — match shadow tone/blur and gradient treatments. Never pure-black shadows.\n\nImprove: layout hierarchy, spacing scale discipline, focus rings, one signature element specific to the new subject. Do NOT copy the reference's content, wording, or layout structure — build a stronger structure from the design knowledge base.\n");
        out
    }
}

pub async fn fetch_reference(url: &str) -> Option<ReferenceSite> {
    let client = build_client();
    let html   = client.get(url).send().await.ok()?.text().await.ok()?;

    // Extract everything with no awaits held over ElementRef.
    let (title, description, headings, body_snippet, inline_css, ext_urls, images) = {
        let doc = Html::parse_document(&html);
        let title_sel = Selector::parse("title").ok()?;
        let meta_sel  = Selector::parse(r#"meta[name="description"]"#).ok()?;
        let h_sel     = Selector::parse("h1, h2, h3").ok()?;
        let p_sel     = Selector::parse("p").ok()?;
        let style_sel = Selector::parse("style").ok()?;
        let link_sel  = Selector::parse("link[rel='stylesheet']").ok()?;
        let img_sel   = Selector::parse("img").ok()?;
        let bg_sel    = Selector::parse("[style*='background-image']").ok()?;

        let title = doc.select(&title_sel).next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let description = doc.select(&meta_sel).next()
            .and_then(|e| e.value().attr("content"))
            .unwrap_or("").to_string();

        let headings: Vec<String> = doc.select(&h_sel)
            .take(12)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty() && s.len() < 120)
            .collect();

        let body_snippet: String = doc.select(&p_sel)
            .take(4)
            .map(|e| e.text().collect::<String>().trim().to_string())
            .filter(|s| s.len() > 24 && s.len() < 240)
            .collect::<Vec<_>>()
            .join(" · ");

        let mut inline_css = String::new();
        for s in doc.select(&style_sel) {
            inline_css.push_str(&s.text().collect::<String>());
        }
        let mut ext_urls: Vec<String> = Vec::new();
        for link in doc.select(&link_sel).take(3) {
            if let Some(href) = link.value().attr("href") {
                ext_urls.push(resolve_url(url, href));
            }
        }

        // ── Image extraction ──
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut images: Vec<ImageAsset> = Vec::new();

        // <img> tags — prefer the largest source in srcset when present.
        for img in doc.select(&img_sel) {
            let src    = img.value().attr("src").unwrap_or("");
            let srcset = img.value().attr("srcset").unwrap_or("");
            let alt    = img.value().attr("alt").unwrap_or("").trim().to_string();
            let w      = img.value().attr("width").and_then(|s| s.parse::<u32>().ok());
            let h      = img.value().attr("height").and_then(|s| s.parse::<u32>().ok());

            let best = if !srcset.is_empty() {
                largest_from_srcset(srcset).unwrap_or_else(|| src.to_string())
            } else {
                src.to_string()
            };
            if best.is_empty() { continue; }
            let absolute = resolve_url(url, &best);
            if is_junk_image(&absolute, w, h) { continue; }
            if !seen.insert(absolute.clone()) { continue; }
            images.push(ImageAsset { url: absolute, alt, w, h });
            if images.len() >= 18 { break; }
        }

        // Inline style background-image URLs.
        for el in doc.select(&bg_sel) {
            let style = el.value().attr("style").unwrap_or("");
            for bg_url in extract_bg_urls(style) {
                let absolute = resolve_url(url, &bg_url);
                if is_junk_image(&absolute, None, None) { continue; }
                if !seen.insert(absolute.clone()) { continue; }
                images.push(ImageAsset { url: absolute, alt: String::new(), w: None, h: None });
                if images.len() >= 18 { break; }
            }
            if images.len() >= 18 { break; }
        }

        (title, description, headings, body_snippet, inline_css, ext_urls, images)
    };

    // Also mine background-image URLs from external / inline CSS after we
    // fetched them below. Do that after CSS accumulation.
    let mut css = inline_css;
    for ext in ext_urls {
        if let Ok(resp) = client.get(&ext).send().await {
            if let Ok(t) = resp.text().await { css.push_str(&t); }
        }
    }

    let mut images = images;
    let mut seen: std::collections::HashSet<String> = images.iter().map(|a| a.url.clone()).collect();
    for bg_url in extract_bg_urls(&css) {
        let absolute = resolve_url(url, &bg_url);
        if is_junk_image(&absolute, None, None) { continue; }
        if !seen.insert(absolute.clone()) { continue; }
        images.push(ImageAsset { url: absolute, alt: String::new(), w: None, h: None });
        if images.len() >= 24 { break; }
    }

    // Cap total images passed to the LLM to keep tokens reasonable.
    if images.len() > 16 { images.truncate(16); }

    // Fire-and-wait a headless-Chrome screenshot in parallel with the extract.
    let screenshot = capture_screenshot(url).await;
    let (has_threejs, has_webgl) = detect_3d_usage(&html, &css);

    Some(ReferenceSite {
        url: url.to_string(),
        title,
        description,
        colors:      extract_hex_colors(&css),
        fonts:       extract_fonts(&css),
        headings,
        body_snippet,
        images,
        animations:  extract_animations(&css),
        transitions: extract_transitions(&css),
        gradients:   extract_gradients(&css),
        shadows:     extract_shadows(&css),
        radii:       extract_radii(&css),
        keyframes:   extract_keyframes(&css),
        screenshot,
        has_threejs,
        has_webgl,
        keyframe_blocks: extract_keyframe_blocks(&css),
        inline_svgs:     extract_inline_svgs(&html),
    })
}

/// Extract full `@keyframes name { ... }` blocks including bodies. Balances
/// braces manually so nested `{}` (function bodies inside from/to steps) are
/// preserved. Caps each block at 600 chars and total to 5 blocks so the
/// prompt doesn't explode.
fn extract_keyframe_blocks(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = css.as_bytes();
    let needle = b"@keyframes";
    let mut i = 0;
    while i + needle.len() < bytes.len() && out.len() < 5 {
        if bytes[i..].starts_with(needle) {
            // Find the opening brace after the name.
            let mut j = i + needle.len();
            while j < bytes.len() && bytes[j] != b'{' { j += 1; }
            if j >= bytes.len() { break; }
            // Walk brace pairs.
            let start = i;
            let mut depth = 1;
            j += 1;
            while j < bytes.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            if depth == 0 {
                let raw = &css[start..j];
                // Normalise whitespace to keep prompt tight.
                let compact = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                let trimmed = if compact.len() > 600 { compact[..600].to_string() + "…" } else { compact };
                out.push(trimmed);
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Extract inline SVG markup from the raw HTML. Limits to 4 SVGs, each up to
/// 900 chars, prioritising ones with any `<path>` (real illustrations) over
/// tiny 1-path icons.
fn extract_inline_svgs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = html.as_bytes();
    let start_tag = b"<svg";
    let end_tag = b"</svg>";
    let mut i = 0;
    while i + start_tag.len() < bytes.len() && out.len() < 6 {
        if bytes[i..].starts_with(start_tag) {
            // Find closing </svg>.
            let start = i;
            let mut j = i + start_tag.len();
            while j + end_tag.len() <= bytes.len() && !bytes[j..].starts_with(end_tag) {
                j += 1;
            }
            if j + end_tag.len() > bytes.len() { break; }
            j += end_tag.len();
            let raw = &html[start..j];
            // Skip trivial icons (1 path or under 200 chars).
            let path_count = raw.matches("<path").count();
            if raw.len() >= 200 && path_count >= 1 {
                let compact = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                let trimmed = if compact.len() > 900 { compact[..900].to_string() + "…</svg>" } else { compact };
                out.push(trimmed);
            }
            i = j;
        } else {
            i += 1;
        }
    }
    // Keep just the 4 largest — they're likelier to be real illustrations.
    out.sort_by(|a, b| b.len().cmp(&a.len()));
    out.truncate(4);
    out
}

/// Detect whether the reference uses three.js or bare WebGL / canvas so we can
/// tell the LLM to include a matching 3D signature element.
fn detect_3d_usage(html: &str, css: &str) -> (bool, bool) {
    let h = html.to_lowercase();
    let c = css.to_lowercase();
    let three_markers = [
        "three.min.js", "three.module.js", "three@0.", "threejs",
        "import * as three", "from \"three\"", "from 'three'",
        "react-three-fiber", "@react-three/", "splinetool",
        "unpkg.com/three", "cdn.jsdelivr.net/npm/three",
    ];
    let webgl_markers = [
        "getcontext(\"webgl", "getcontext('webgl",
        "getcontext(\"experimental-webgl", "getcontext('experimental-webgl",
        "webglrenderer", "requestanimationframe",
    ];
    let has_threejs = three_markers.iter().any(|m| h.contains(m) || c.contains(m));
    let has_canvas  = h.contains("<canvas");
    let has_webgl   = has_canvas && webgl_markers.iter().any(|m| h.contains(m) || c.contains(m));
    (has_threejs, has_webgl && !has_threejs)
}

fn extract_animations(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)\banimation\s*:\s*([^;{}]+)") { Ok(r) => r, Err(_) => return vec![] };
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty() && s.len() < 120 && seen.insert(s.clone()))
        .take(8).collect()
}

fn extract_transitions(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)\btransition\s*:\s*([^;{}]+)") { Ok(r) => r, Err(_) => return vec![] };
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty() && s.len() < 100 && seen.insert(s.clone()))
        .take(10).collect()
}

fn extract_gradients(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)\b(?:linear|radial|conic)-gradient\s*\([^)]{4,180}\)") {
        Ok(r) => r, Err(_) => return vec![]
    };
    let mut seen = std::collections::HashSet::new();
    re.find_iter(css)
        .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| seen.insert(s.clone()))
        .take(6).collect()
}

fn extract_shadows(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)\bbox-shadow\s*:\s*([^;{}]{5,180})") {
        Ok(r) => r, Err(_) => return vec![]
    };
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .take(6).collect()
}

fn extract_radii(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)\bborder-radius\s*:\s*([^;{}]{1,80})") {
        Ok(r) => r, Err(_) => return vec![]
    };
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .take(8).collect()
}

fn extract_keyframes(css: &str) -> Vec<String> {
    let re = match Regex::new(r"(?i)@keyframes\s+([A-Za-z_][A-Za-z0-9_-]*)") {
        Ok(r) => r, Err(_) => return vec![]
    };
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|s| seen.insert(s.clone()))
        .take(10).collect()
}

/// Best-effort headless-Chrome screenshot of the reference URL. Tries known
/// Chrome binaries on macOS; returns None if none found or the run fails.
/// Times out at 15 s so a broken reference doesn't hang generation.
pub async fn capture_screenshot(url: &str) -> Option<std::path::PathBuf> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    ];
    let chrome = candidates.iter().find(|p| std::path::Path::new(p).exists())?;

    let temp_dir = std::env::temp_dir().join("design-gen-refs");
    std::fs::create_dir_all(&temp_dir).ok()?;
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let out_path = temp_dir.join(format!("ref-{}.png", ts));

    let chrome = chrome.to_string();
    let url = url.to_string();
    let out_clone = out_path.clone();

    let handle = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&chrome)
            .args([
                "--headless=new",
                "--disable-gpu",
                "--hide-scrollbars",
                "--no-sandbox",
                "--window-size=1440,900",
                "--virtual-time-budget=8000",
                &format!("--screenshot={}", out_clone.to_string_lossy()),
                &url,
            ])
            .output()
    });

    let output = tokio::time::timeout(std::time::Duration::from_secs(15), handle).await.ok()?.ok()?.ok()?;
    if output.status.success() && out_path.exists() {
        Some(out_path)
    } else {
        None
    }
}

/// Given a srcset like "a.jpg 400w, b.jpg 800w" or "a.jpg 1x, b.jpg 2x",
/// return the URL with the largest descriptor.
fn largest_from_srcset(srcset: &str) -> Option<String> {
    let mut best: Option<(f64, String)> = None;
    for candidate in srcset.split(',') {
        let mut parts = candidate.trim().split_whitespace();
        let url = parts.next()?.to_string();
        let descriptor = parts.next().unwrap_or("1x");
        let val = descriptor
            .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.')
            .parse::<f64>()
            .unwrap_or(1.0);
        if best.as_ref().map(|(v, _)| val > *v).unwrap_or(true) {
            best = Some((val, url));
        }
    }
    best.map(|(_, u)| u)
}

/// Extract url(...) values from CSS `background-image` declarations.
fn extract_bg_urls(css: &str) -> Vec<String> {
    let re = match Regex::new(r#"(?i)background(?:-image)?\s*:\s*[^;{}]*url\(\s*['"]?([^'"\)\s]+)"#) {
        Ok(r) => r, Err(_) => return vec![],
    };
    re.captures_iter(css)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .filter(|u| !u.starts_with("data:"))
        .collect()
}

/// Filter out tracking pixels, tiny icons, and data URLs.
fn is_junk_image(url: &str, w: Option<u32>, h: Option<u32>) -> bool {
    let lower = url.to_lowercase();
    if lower.starts_with("data:") { return true; }
    if lower.is_empty()           { return true; }
    // Common tracking / analytics markers in the path.
    for marker in &["pixel", "beacon", "track", "google-analytics", "doubleclick", "hotjar", "segment.io", "sentry"] {
        if lower.contains(marker) { return true; }
    }
    // Extremely tiny declared dims → tracking pixel / spacer.
    if let (Some(w), Some(h)) = (w, h) {
        if w <= 8 || h <= 8 { return true; }
    }
    false
}

async fn extract_css_tokens(client: &Client, url: &str) -> Result<(Vec<String>, Vec<String>)> {
    let html = client.get(url).send().await?.text().await?;

    // Collect all CSS + external sheet URLs synchronously before any more awaits
    let (inline_css, ext_urls) = {
        let doc = Html::parse_document(&html);
        let mut inline = String::new();
        let mut urls: Vec<String> = vec![];

        for s in doc.select(&Selector::parse("style").unwrap()) {
            inline.push_str(&s.text().collect::<String>());
        }
        for link in doc.select(&Selector::parse("link[rel='stylesheet']").unwrap()).take(2) {
            if let Some(href) = link.value().attr("href") {
                urls.push(resolve_url(url, href));
            }
        }
        (inline, urls)
        // doc, selectors, ElementRefs all dropped here — before any await
    };

    let mut css = inline_css;
    for ext_url in ext_urls {
        if let Ok(resp) = client.get(&ext_url).send().await {
            if let Ok(text) = resp.text().await {
                css.push_str(&text);
            }
        }
    }

    Ok((extract_hex_colors(&css), extract_fonts(&css)))
}

fn extract_hex_colors(css: &str) -> Vec<String> {
    let re = Regex::new(r"#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})\b").unwrap();
    let mut seen = std::collections::HashSet::new();
    re.find_iter(css)
        .map(|m| m.as_str().to_uppercase())
        .filter(|c| seen.insert(c.clone()))
        .take(6).collect()
}

fn extract_fonts(css: &str) -> Vec<String> {
    let re = Regex::new(r#"font-family\s*:\s*([^;}{]+)"#).unwrap();
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(css)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split(',').next().unwrap_or("").trim()
                  .trim_matches('"').trim_matches('\'').to_string())
        .filter(|f| !f.is_empty() && seen.insert(f.clone()))
        .take(3).collect()
}

fn resolve_url(base: &str, href: &str) -> String {
    if href.starts_with("http") { return href.to_string(); }
    if href.starts_with("//")   { return format!("https:{}", href); }
    if let Ok(b) = url::Url::parse(base) {
        if let Ok(u) = b.join(href) { return u.to_string(); }
    }
    href.to_string()
}
