use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use tokio::sync::mpsc as tokio_mpsc;
use std::sync::{Arc, mpsc::Sender, atomic::{AtomicBool, Ordering}};

use crate::ai::{self, Provider};
use crate::knowledge::{DesignEntry, KnowledgeBase};
use crate::projects;
use crate::scraper;
use crate::variants::{self, Variant, Palette, Theme};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct IpcMessage {
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    AssistantMessage(String),
    StatusUpdate(String),
    DesignUpdate(String),
    ExportDesign(String),
    ExportPrototype(String),
    ThinkingChunk(String),
    SetGenerating(bool),
    ProjectsList(String),
    ProjectOpened { slug: String, name: String, html: String, chat: String },
    Meta { provider: String, model: String },
    /// Interim preview during progressive assembly — updates the iframe only,
    /// does NOT touch history, state, or the "ready" pill.
    AssemblyPreview(String),
    CritiqueFixes(String), // JSON array [{label, prompt}]
    TokenUsage {
        turn_input:     u32,
        turn_output:    u32,
        session_input:  u64,
        session_output: u64,
        estimated:      bool,
    },
}

enum State {
    AwaitingIdea,
    Refining {
        current: String,
        idea:    String,
        theme:   String,
        tried_archetypes: Vec<String>,
    },
}

pub async fn run(
    mut rx: tokio_mpsc::UnboundedReceiver<IpcMessage>,
    tx: Sender<AppEvent>,
    provider: Provider,
    provider_id: String,
    model: String,
    stop_flag: Arc<AtomicBool>,
) {
    let mut state           = State::AwaitingIdea;
    let mut knowledge       = KnowledgeBase::load();
    let mut current_project: Option<String> = None;
    let mut session_input:  u64 = 0;
    let mut session_output: u64 = 0;

    let send_meta = |tx: &Sender<AppEvent>| {
        let _ = tx.send(AppEvent::Meta { provider: provider_id.clone(), model: model.clone() });
    };

    send_meta(&tx);
    push_projects_list(&tx);

    while let Some(msg) = rx.recv().await {
        match msg.kind.as_str() {
            "export"           => { let _ = tx.send(AppEvent::ExportDesign(msg.content));   continue; }
            "export_prototype" => { let _ = tx.send(AppEvent::ExportPrototype(msg.content)); continue; }

            "list_projects" => { send_meta(&tx); push_projects_list(&tx); continue; }

            "create_project" => {
                match projects::create(msg.content.trim()) {
                    Ok(p) => {
                        current_project = Some(p.slug.clone());
                        state = State::AwaitingIdea;
                        let _ = tx.send(AppEvent::ProjectOpened {
                            slug: p.slug.clone(),
                            name: p.name.clone(),
                            html: String::new(),
                            chat: "[]".into(),
                        });
                        push_projects_list(&tx);
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Could not create project: {}", e)
                        ));
                    }
                }
                continue;
            }

            "open_project" => {
                let slug = msg.content.trim().to_string();
                match projects::read(&slug) {
                    Ok(html) => {
                        current_project = Some(slug.clone());
                        let name = projects::name_of(&slug).unwrap_or_default();
                        let chat = projects::read_chat(&slug).unwrap_or_else(|_| "[]".into());
                        if !html.trim().is_empty() {
                            state = State::Refining {
                                current: html.clone(),
                                idea:    name.clone(),
                                theme:   String::new(),
                                tried_archetypes: Vec::new(),
                            };
                        } else {
                            state = State::AwaitingIdea;
                        }
                        let _ = tx.send(AppEvent::ProjectOpened { slug, name, html, chat });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Could not open project: {}", e)
                        ));
                    }
                }
                continue;
            }

            "save_chat" => {
                if let Some(ref slug) = current_project {
                    let _ = projects::write_chat(slug, &msg.content);
                }
                continue;
            }

            "delete_project" => {
                let slug = msg.content.trim();
                let _ = projects::delete(slug);
                if current_project.as_deref() == Some(slug) {
                    current_project = None;
                    state = State::AwaitingIdea;
                }
                push_projects_list(&tx);
                continue;
            }

            "start_design" => {
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let idea  = payload["idea"].as_str().unwrap_or("").trim().to_string();
                let theme = payload["theme"].as_str().unwrap_or("auto").trim().to_string();
                if idea.is_empty() { continue; }

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));

                let result = handle_start_design(
                    &idea, &theme, &[],
                    &provider, &tx, stop_flag.clone(), &knowledge,
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        persist_current(&current_project, &state, &tx);
                        if let State::Refining { current, .. } = &state {
                            spawn_critique(current.clone(), provider.clone(), tx.clone());
                        }
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Something went wrong:\n{}\n\nTry again.", e)
                        ));
                    }
                }
                continue;
            }

            "assemble_design" => {
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let idea    = payload["idea"].as_str().unwrap_or("").trim().to_string();
                let theme   = payload["theme"].as_str().unwrap_or("").to_string();
                let palette = payload["palette"].as_str().unwrap_or("").to_string();

                let mut picks: HashMap<String, String> = HashMap::new();
                for cat in ["navbar","hero","features","testimonials","pricing","cta","footer"] {
                    if let Some(v) = payload[cat].as_str() {
                        if !v.is_empty() && v != "auto" {
                            picks.insert(cat.into(), v.into());
                        }
                    }
                }

                if idea.is_empty() { continue; }

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));

                let result = handle_assemble(
                    &idea, &theme, &palette, &picks,
                    &provider, &tx, stop_flag.clone(),
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        persist_current(&current_project, &state, &tx);
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Couldn't assemble your design:\n{}", e)
                        ));
                    }
                }
                continue;
            }

            "swap_section" => {
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let variant_id = payload["variant"].as_str().unwrap_or("").to_string();
                if variant_id.is_empty() { continue; }

                let (current, idea, theme_id, tried) = match &state {
                    State::Refining { current, idea, theme, tried_archetypes } =>
                        (current.clone(), idea.clone(), theme.clone(), tried_archetypes.clone()),
                    _ => { continue; }
                };

                match swap_section_in_html(&current, &variant_id) {
                    Ok(new_html) => {
                        state = State::Refining {
                            current: new_html.clone(),
                            idea, theme: theme_id,
                            tried_archetypes: tried,
                        };
                        let _ = tx.send(AppEvent::DesignUpdate(new_html));
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Swapped to {}.", variant_id)
                        ));
                        persist_current(&current_project, &state, &tx);
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Swap failed: {}", e)
                        ));
                    }
                }
                continue;
            }

            "try_different_layout" => {
                let (idea, theme, tried) = match &state {
                    State::Refining { idea, theme, tried_archetypes, .. } =>
                        (idea.clone(), theme.clone(), tried_archetypes.clone()),
                    _ => { continue; }
                };
                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));

                let result = handle_start_design(
                    &idea, &theme, &tried,
                    &provider, &tx, stop_flag.clone(), &knowledge,
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        persist_current(&current_project, &state, &tx);
                        if let State::Refining { current, .. } = &state {
                            spawn_critique(current.clone(), provider.clone(), tx.clone());
                        }
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Different-layout attempt failed:\n{}", e)
                        ));
                    }
                }
                continue;
            }

            "refine_element" => {
                let (current, idea, theme, tried) = match &state {
                    State::Refining { current, idea, theme, tried_archetypes } =>
                        (current.clone(), idea.clone(), theme.clone(), tried_archetypes.clone()),
                    _ => { continue; }
                };
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let selector = payload["selector"].as_str().unwrap_or("").to_string();
                let outer    = payload["outer_html"].as_str().unwrap_or("").to_string();
                let prompt   = payload["prompt"].as_str().unwrap_or("").to_string();
                if outer.is_empty() || prompt.is_empty() { continue; }

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));

                let result = handle_refine_element(
                    &selector, &outer, &prompt, &current, &idea, &theme, &tried,
                    &provider, &tx, stop_flag.clone(),
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        persist_current(&current_project, &state, &tx);
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Element edit failed:\n{}", e)
                        ));
                    }
                }
                continue;
            }

            "save_design" => {
                if let State::Refining { ref current, ref idea, ref theme, .. } = state {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs()).unwrap_or(0);
                    knowledge.add(DesignEntry {
                        idea: idea.clone(), tone: theme.clone(),
                        colors: extract_colors(current),
                        fonts:  extract_fonts(current),
                        timestamp: ts,
                    });
                    let _ = tx.send(AppEvent::AssistantMessage(
                        "✓ Design pattern saved to knowledge base.".into()
                    ));
                }
                continue;
            }

            "sync_design" => {
                if let State::Refining { ref mut current, .. } = state {
                    *current = msg.content.clone();
                }
                if let Some(ref slug) = current_project {
                    let _ = projects::write(slug, &msg.content);
                }
                continue;
            }

            "stop_generation" => {
                stop_flag.store(true, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(false));
                let _ = tx.send(AppEvent::AssistantMessage("Generation stopped.".into()));
                continue;
            }

            "user_message" => {}
            _ => continue,
        }

        // user_message routing — refine only. Fresh idea/theme flow is start_design.
        let content = msg.content.trim().to_string();
        if content.is_empty() { continue; }

        let result = match &state {
            State::Refining { current, idea, theme, tried_archetypes } => {
                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));
                let r = handle_refine(
                    &content, current, idea, theme, tried_archetypes,
                    &provider, &tx, stop_flag.clone(), &knowledge,
                ).await;
                let _ = tx.send(AppEvent::SetGenerating(false));
                r
            }
            State::AwaitingIdea => {
                let _ = tx.send(AppEvent::AssistantMessage(
                    "Pick a theme (chips above) or type a theme description to start.".into()
                ));
                continue;
            }
        };

        match result {
            Ok((next, usage)) => {
                state = next;
                emit_usage(&tx, usage, &mut session_input, &mut session_output);
                persist_current(&current_project, &state, &tx);
            }
            Err(e) if e.to_string() == "__stopped__" => {}
            Err(e) => {
                let _ = tx.send(AppEvent::AssistantMessage(
                    format!("Something went wrong:\n{}\n\nPlease try again.", e)
                ));
            }
        }
    }
}

fn emit_usage(
    tx: &Sender<AppEvent>, usage: ai::Usage,
    session_input: &mut u64, session_output: &mut u64,
) {
    *session_input  = session_input.saturating_add(usage.input_tokens  as u64);
    *session_output = session_output.saturating_add(usage.output_tokens as u64);
    let _ = tx.send(AppEvent::TokenUsage {
        turn_input:     usage.input_tokens,
        turn_output:    usage.output_tokens,
        session_input:  *session_input,
        session_output: *session_output,
        estimated:      usage.estimated,
    });
}

fn persist_current(current_project: &Option<String>, state: &State, tx: &Sender<AppEvent>) {
    if let (Some(slug), State::Refining { current, .. }) = (current_project.as_ref(), state) {
        let _ = projects::write(slug, current);
        push_projects_list(tx);
    }
}

fn push_projects_list(tx: &Sender<AppEvent>) {
    let items = projects::list().unwrap_or_default();
    let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
    let _ = tx.send(AppEvent::ProjectsList(json));
}

async fn stream_generate(
    provider: &Provider, system: &str, user: &str, max_tokens: u32,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
) -> Result<ai::Completion> {
    let tx2 = tx.clone();
    // The SYSTEM_CONTEXT (design knowledge + image toolkit) is a big static
    // prefix identical across every call. Sending it as a cacheable block
    // enables Anthropic prompt caching and OpenAI's implicit caching.
    let comp = provider.complete_streaming_cached(
        system, ai::prompts::SYSTEM_CONTEXT, user, max_tokens,
        Box::new(move |chunk| { let _ = tx2.send(AppEvent::ThinkingChunk(chunk)); }),
        stop,
    ).await?;
    Ok(ai::Completion { text: ai::clean(comp.text), usage: comp.usage })
}

fn usage_or_estimate(usage: Option<ai::Usage>, system: &str, user: &str, output: &str) -> ai::Usage {
    match usage {
        Some(u) => u,
        None => ai::Usage {
            input_tokens:  ai::estimate_tokens(system.len() + user.len()),
            output_tokens: ai::estimate_tokens(output.len()),
            estimated:     true,
        },
    }
}

/// Pull the first http/https URL out of a chat message. Trailing punctuation
/// is stripped so `"like https://acme.co."` yields `https://acme.co`.
fn extract_first_url(text: &str) -> Option<String> {
    let re = Regex::new(r"https?://[^\s<>\)\]\}]+").ok()?;
    re.find(text).map(|m| {
        m.as_str()
            .trim_end_matches(|c: char| ".,;:!?)]}\"'".contains(c))
            .to_string()
    })
}

/// Extract archetype name from a `<meta name="archetype" content="X">` tag if present.
fn extract_archetype(html: &str) -> Option<String> {
    let re = Regex::new(r#"(?i)<meta\s+name=["']archetype["']\s+content=["']([^"']+)["']"#).ok()?;
    let cap = re.captures(html)?;
    Some(cap.get(1)?.as_str().trim().to_lowercase())
}

async fn handle_start_design(
    idea: &str, theme: &str, tried: &[String],
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
    knowledge: &KnowledgeBase,
) -> Result<(State, ai::Usage)> {
    // If the user pasted a URL in the idea, fetch it as a reference site so
    // the LLM can "make something better" than it.
    let user_ref_block = if let Some(u) = extract_first_url(idea) {
        let _ = tx.send(AppEvent::StatusUpdate(format!("Fetching reference: {}…", u)));
        match scraper::fetch_reference(&u).await {
            Some(rs) => rs.as_prompt_block(),
            None => String::new(),
        }
    } else { String::new() };

    // Skip web scraping unless the user explicitly pasted a reference URL. For
    // non-landing-page ideas (dashboards, portfolios, admin panels) awwwards
    // results are irrelevant and bias the LLM toward marketing sites, plus the
    // extra scraper output pushes the CLI prompt into slow/hang territory.
    let refs = if user_ref_block.is_empty() {
        String::new()
    } else {
        let _ = tx.send(AppEvent::StatusUpdate("Scraping design references...".into()));
        scraper::gather(idea).await
    };
    let _ = tx.send(AppEvent::StatusUpdate(
        if tried.is_empty() { "Designing…".into() } else { "Redesigning with a different layout…".into() }
    ));

    // Prepend the user's chosen reference (if any) to the scraped refs block —
    // instructions in the reference block tell the LLM to treat it as a
    // quality benchmark and NOT to copy content.
    let combined_refs = if user_ref_block.is_empty() {
        refs
    } else {
        format!("{user_ref_block}\n\n{refs}")
    };

    let excluded = tried.join(", ");
    let system = ai::prompts::SKELETON_SINGLE_STYLED_SYSTEM;
    let user   = ai::prompts::skeleton_single_styled_user(idea, theme, &excluded, &combined_refs, &knowledge.prompt_context());
    let comp = stream_generate(provider, system, &user, 8000, tx, stop).await?;
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let archetype = extract_archetype(&comp.text).unwrap_or_else(|| "unknown".into());
    let mut all_tried = tried.to_vec();
    if !all_tried.contains(&archetype) { all_tried.push(archetype.clone()); }

    let _ = tx.send(AppEvent::DesignUpdate(comp.text.clone()));
    let _ = tx.send(AppEvent::AssistantMessage(format!(
        "Design ready — {} layout. Edit directly on the canvas, refine here, or ask for a different layout.",
        archetype
    )));

    Ok((
        State::Refining {
            current: comp.text,
            idea:    idea.to_string(),
            theme:   theme.to_string(),
            tried_archetypes: all_tried,
        },
        usage,
    ))
}

async fn handle_refine(
    feedback: &str, current: &str, idea: &str, theme: &str, tried: &[String],
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
    knowledge: &KnowledgeBase,
) -> Result<(State, ai::Usage)> {
    let _ = tx.send(AppEvent::StatusUpdate("Refining design...".into()));
    let system  = ai::prompts::REFINE_SYSTEM;
    let bounded = bound_html(current, 40_000);
    let user    = ai::prompts::refine_user(&bounded, feedback, &knowledge.prompt_context());
    let comp = stream_generate(provider, system, &user, 6000, tx, stop).await?;
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let _ = tx.send(AppEvent::DesignUpdate(comp.text.clone()));
    let _ = tx.send(AppEvent::AssistantMessage("Updated. Anything else?".into()));

    Ok((
        State::Refining {
            current: comp.text,
            idea:    idea.to_string(),
            theme:   theme.to_string(),
            tried_archetypes: tried.to_vec(),
        },
        usage,
    ))
}

async fn handle_refine_element(
    selector: &str, outer_html: &str, feedback: &str,
    current: &str, idea: &str, theme: &str, tried: &[String],
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
) -> Result<(State, ai::Usage)> {
    let _ = tx.send(AppEvent::StatusUpdate(
        format!("Refining {}...", if selector.is_empty() { "element" } else { selector })
    ));
    let system = ai::prompts::ELEMENT_REFINE_SYSTEM;
    let user   = ai::prompts::element_refine_user(selector, outer_html, feedback);
    let comp   = stream_generate(provider, system, &user, 3000, tx, stop).await?;
    let usage  = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let new_element = comp.text.trim().to_string();
    if new_element.is_empty() {
        return Err(anyhow::anyhow!("empty replacement returned"));
    }

    let refined = match splice_element(current, outer_html, &new_element) {
        Some(s) => s,
        None => {
            let _ = tx.send(AppEvent::AssistantMessage(
                "Couldn't find the exact element in the source — the canvas may have been edited since selection. Try selecting the element again.".into()
            ));
            return Ok((
                State::Refining {
                    current: current.to_string(), idea: idea.to_string(),
                    theme: theme.to_string(), tried_archetypes: tried.to_vec(),
                },
                usage,
            ));
        }
    };

    let _ = tx.send(AppEvent::DesignUpdate(refined.clone()));
    let _ = tx.send(AppEvent::AssistantMessage(
        format!("Updated {}. Anything else?", if selector.is_empty() { "the element" } else { selector })
    ));

    Ok((
        State::Refining {
            current: refined, idea: idea.to_string(),
            theme: theme.to_string(), tried_archetypes: tried.to_vec(),
        },
        usage,
    ))
}

fn splice_element(current: &str, outer_html: &str, replacement: &str) -> Option<String> {
    let anchor = outer_html.trim();
    if anchor.is_empty() { return None; }
    let pos = current.find(anchor)?;
    let mut out = String::with_capacity(current.len() + replacement.len());
    out.push_str(&current[..pos]);
    out.push_str(replacement);
    out.push_str(&current[pos + anchor.len()..]);
    Some(out)
}

fn bound_html(html: &str, max_chars: usize) -> String {
    if html.len() <= max_chars { return html.to_string(); }
    let head = &html[..max_chars * 2 / 3];
    let tail = &html[html.len() - max_chars / 3..];
    format!("{head}\n<!-- middle truncated -->\n{tail}")
}

/// Assemble a design from user-picked variants + palette + theme. Runs a
/// small LLM content-fill call to populate placeholders, then splices the
/// filled variants into a full HTML shell.
async fn handle_assemble(
    idea: &str, theme_id: &str, palette_id: &str,
    picks: &HashMap<String, String>,
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
) -> Result<(State, ai::Usage)> {
    let lib = variants::library();

    let theme = lib.themes.get(theme_id)
        .or_else(|| lib.themes.get("editorial-warm-cream"))
        .ok_or_else(|| anyhow::anyhow!("no themes available"))?;

    let palette_key = if !palette_id.is_empty() && lib.palettes.contains_key(palette_id) {
        palette_id.to_string()
    } else {
        theme.meta.get("palette").cloned().unwrap_or_else(|| "warm-cream-brick".into())
    };
    let palette = lib.palettes.get(&palette_key)
        .or_else(|| lib.palettes.get("warm-cream-brick"))
        .ok_or_else(|| anyhow::anyhow!("no palettes available"))?;

    // Resolve variant for each category — user pick, else first available.
    let categories = ["navbar", "hero", "features", "testimonials", "pricing", "cta", "footer"];
    let mut selected: Vec<&Variant> = Vec::new();
    for cat in &categories {
        let variant = picks.get(*cat)
            .and_then(|id| lib.variants.get(id))
            .or_else(|| lib.variants_in(cat).into_iter().next());
        if let Some(v) = variant { selected.push(v); }
    }
    if selected.is_empty() {
        return Err(anyhow::anyhow!("variant library empty — no sections to assemble"));
    }

    // Collect all placeholders across chosen variants.
    let mut all_placeholders: Vec<String> = Vec::new();
    for v in &selected {
        for p in &v.placeholders {
            if !all_placeholders.contains(p) { all_placeholders.push(p.clone()); }
        }
    }

    let _ = tx.send(AppEvent::StatusUpdate(
        format!("Filling {} placeholders for \"{}\"…", all_placeholders.len(), idea)
    ));

    let system = "You are a UX copywriter filling placeholders for a web interface (could be a landing page, dashboard, product UI, admin panel, portfolio, or any other type). \
Output STRICTLY a JSON object mapping each placeholder key (uppercase, underscored) \
to a concrete filled string. Follow these rules:\n\
- Use specific nouns from the subject's world, never generic benefit words.\n\
- Headlines: 8-14 words, ONE concrete noun the audience recognizes.\n\
- Subheads: 12-24 words, elaborate the specific benefit.\n\
- CTAs: 2-4 words, verb-noun, specific first action (\"Book 20 min\", \"Import invoices\"), NEVER \"Get started\".\n\
- Stats: specific odd numbers, not round (\"4 hours saved per week — median\"), never \"10x\" or \"99.9%\".\n\
- Testimonials: named person + realistic role + realistic company + specific outcome number.\n\
- Image URLs: use https://loremflickr.com/{W}/{H}/{keywords}?lock={seed} matched to subject, OR https://i.pravatar.cc/{size}?img={1..70} for avatars.\n\
- Feature titles: 2-4 words, concrete product noun.\n\
- Use realistic email placeholders (e.g. \"you@studio.co\").\n\
No prose. No markdown fences. Just a JSON object.";

    let user = format!(
        "Subject: {idea}\nTone package: {theme_id}\n\nFill these placeholder keys with concrete values suitable for \"{idea}\":\n\n{}\n\nReturn a JSON object like {{\"KEY\": \"value\", ...}}. Every key above must appear once.",
        all_placeholders.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n"),
    );

    // Assembly LLM outputs JSON, not HTML — DON'T stream chunks to the
    // preview iframe or the right pane will show raw text mid-flight.
    let comp = provider.complete_streaming_cached(
        system, ai::prompts::SYSTEM_CONTEXT, &user, 3500,
        Box::new(|_| {}),
        stop,
    ).await?;
    let comp = ai::Completion { text: ai::clean(comp.text), usage: comp.usage };
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let fills = parse_placeholder_fills(&comp.text, &all_placeholders);

    // ── Progressive assembly reveal ──
    // Instead of showing the completed HTML in one flash, add sections one at
    // a time to the preview iframe. Each step is a full HTML snapshot with a
    // growing <body>. Small delays make the build feel intentional.
    let (shell_open, shell_close) = assemble_shell(theme, palette, &selected, idea);
    // Frame 0: empty styled shell — user sees the palette + typography land.
    let empty = format!("{shell_open}{shell_close}");
    let _ = tx.send(AppEvent::AssemblyPreview(empty.clone()));
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let mut body_acc = String::new();
    for v in &selected {
        let _ = tx.send(AppEvent::StatusUpdate(
            format!("Placing {}…", pretty_category(&v.category))
        ));
        let mut section_html = v.html.clone();
        for p in &v.placeholders {
            let key = format!("{{{{{}}}}}", p);
            let val = fills.get(p).cloned().unwrap_or_else(|| p.replace('_', " ").to_lowercase());
            section_html = section_html.replace(&key, &val);
        }
        body_acc.push_str(&section_html);
        body_acc.push('\n');
        let interim = format!("{shell_open}{body_acc}{shell_close}");
        let _ = tx.send(AppEvent::AssemblyPreview(interim));
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // Final: real DesignUpdate that touches state/history/pill.
    let html = format!("{shell_open}{body_acc}{shell_close}");
    let _ = tx.send(AppEvent::DesignUpdate(html.clone()));
    let _ = tx.send(AppEvent::AssistantMessage(format!(
        "Assembled from {} sections. Click any section on the canvas — a ↻ swap panel opens on the left for zero-token variant swaps.",
        selected.len()
    )));

    Ok((
        State::Refining {
            current: html,
            idea:    idea.to_string(),
            theme:   theme_id.to_string(),
            tried_archetypes: Vec::new(),
        },
        usage,
    ))
}

/// Parse the LLM's JSON dict of placeholder fills, tolerant of markdown fences
/// and unbalanced quoting. Fills in any missing keys with a fallback.
fn parse_placeholder_fills(raw: &str, keys: &[String]) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    let stripped = raw.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    let start = stripped.find('{');
    let end   = stripped.rfind('}');
    if let (Some(a), Some(b)) = (start, end) {
        if b > a {
            let json_str = &stripped[a..=b];
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(obj) = v.as_object() {
                    for (k, val) in obj {
                        let s = val.as_str().map(|s| s.to_string())
                            .unwrap_or_else(|| val.to_string().trim_matches('"').to_string());
                        out.insert(k.clone(), s);
                    }
                }
            }
        }
    }
    // Fallback: any key the LLM missed gets a sensible placeholder.
    for k in keys {
        if !out.contains_key(k) {
            out.insert(k.clone(), fallback_for(k));
        }
    }
    out
}

fn fallback_for(key: &str) -> String {
    let lower = key.to_lowercase();
    // Images / avatars.
    if lower.contains("avatar") {
        return format!("https://i.pravatar.cc/72?img={}", (key.len() % 70) + 1);
    }
    if lower.contains("image") || lower.contains("_url") || lower.ends_with("url") {
        return format!("https://loremflickr.com/1200/675/product?lock={}", key.len());
    }
    // Structured pricing keys.
    if lower.starts_with("t1_") || lower.starts_with("t2_") || lower.starts_with("t3_") {
        if lower.ends_with("_name")    { return match &lower[..2] { "t1" => "Starter", "t2" => "Pro", _ => "Enterprise" }.into(); }
        if lower.ends_with("_price")   { return match &lower[..2] { "t1" => "$0", "t2" => "$29", _ => "Custom" }.into(); }
        if lower.ends_with("_tagline") { return "For your first steps".into(); }
        if lower.ends_with("_cta")     { return "Choose plan".into(); }
        if lower.contains("_f")        { return "Included feature".into(); }
    }
    // Common section keys.
    match lower.as_str() {
        "eyebrow" | "section_eyebrow" => "Overview".into(),
        "headline" | "section_headline" => "Made for real work, not demos".into(),
        "subhead" | "section_subhead" => "Specific words about a specific outcome, from the world your users already recognise.".into(),
        "cta_primary" | "cta_label" => "Try it".into(),
        "cta_secondary" => "See how".into(),
        "trust_line" => "Trusted by 480 studios in 12 cities".into(),
        "brand_name" => "Studio".into(),
        "tagline" => "For people who ship.".into(),
        "copyright" => "© 2026 · All rights reserved".into(),
        "attribution" => "Made by 4 humans in Brooklyn".into(),
        "input_placeholder" => "you@studio.co".into(),
        "guarantee_line" => "30-day money-back guarantee".into(),
        _ => {
            // Title-case fallback for anything else.
            let mut out = String::new();
            for word in key.split('_') {
                if !out.is_empty() { out.push(' '); }
                let mut chars = word.chars();
                if let Some(c) = chars.next() { out.push(c); }
                for c in chars { out.push(c.to_ascii_lowercase()); }
            }
            out
        }
    }
}

fn pretty_category(cat: &str) -> String {
    match cat {
        "navbar" => "navigation".into(),
        "hero"   => "hero".into(),
        "features" => "features".into(),
        "testimonials" => "testimonials".into(),
        "pricing" => "pricing".into(),
        "cta" => "call to action".into(),
        "footer" => "footer".into(),
        other => other.to_string(),
    }
}

/// Split assembly output into (shell_open, shell_close). Body concatenates
/// between them. Used by the progressive assembly reveal.
fn assemble_shell(
    theme: &Theme, palette: &Palette, sections: &[&Variant], idea: &str,
) -> (String, String) {
    let fonts_meta = theme.meta.get("fonts").cloned().unwrap_or_default();
    let display = extract_font(&fonts_meta, "display").unwrap_or_else(|| "Inter".into());
    let body    = extract_font(&fonts_meta, "body").unwrap_or_else(|| "Inter".into());
    let mono    = extract_font(&fonts_meta, "mono").unwrap_or_else(|| "JetBrains Mono".into());

    let mut fonts_needed = vec![display.clone(), body.clone(), mono.clone()];
    fonts_needed.sort(); fonts_needed.dedup();
    let font_families: Vec<String> = fonts_needed.iter()
        .filter(|f| !f.is_empty())
        .map(|f| format!("family={}:ital,wght@0,400;0,500;0,600;0,700;1,400", f.replace(' ', "+")))
        .collect();
    let font_url = if font_families.is_empty() { String::new() }
        else { format!("https://fonts.googleapis.com/css2?{}&display=swap", font_families.join("&")) };

    let mut head = String::new();
    head.push_str("<!DOCTYPE html>\n<html lang=\"en\"><head>\n");
    head.push_str("<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    head.push_str(&format!("<meta name=\"assembled\" content=\"theme:{} palette:{}\">\n", theme.id, palette.id));
    head.push_str(&format!("<title>{}</title>\n", html_escape(idea)));
    if !font_url.is_empty() {
        head.push_str("<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n");
        head.push_str("<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n");
        head.push_str(&format!("<link href=\"{}\" rel=\"stylesheet\">\n", font_url));
    }
    head.push_str("<style>\n");
    head.push_str("*,*::before,*::after{box-sizing:border-box;margin:0;padding:0;}\n");
    head.push_str(":root{\n");
    head.push_str(&palette.body); head.push('\n');
    head.push_str(&format!("--font-display: '{}', Georgia, serif;\n", display));
    head.push_str(&format!("--font-body: '{}', -apple-system, sans-serif;\n", body));
    head.push_str(&format!("--font-mono: '{}', ui-monospace, monospace;\n", mono));
    head.push_str("}\n");
    head.push_str("body{font-family:var(--font-body);color:var(--ink);background:var(--paper);line-height:1.5;-webkit-font-smoothing:antialiased;}\n");
    head.push_str("html{scroll-behavior:smooth;}\n");
    head.push_str("img{max-width:100%;display:block;}\na{color:inherit;}\n");
    for v in sections {
        head.push_str(&v.style);
        head.push('\n');
    }
    head.push_str("@keyframes fadeInUp{from{opacity:0;transform:translateY(16px);}to{opacity:1;transform:translateY(0);}}\n");
    head.push_str("section,nav,footer{animation:fadeInUp 0.4s ease-out both;}\n");
    head.push_str("@media (prefers-reduced-motion: reduce){section,nav,footer{animation:none;}}\n");
    head.push_str("</style>\n</head>\n<body>\n");

    let tail = String::from("</body>\n</html>\n");
    (head, tail)
}

/// Assemble the final HTML: shell + palette + theme fonts + variant styles + variant HTMLs (with placeholders filled).
fn assemble_html(
    theme: &Theme, palette: &Palette,
    sections: &[&Variant], fills: &HashMap<String, String>,
    idea: &str,
) -> String {
    let fonts_meta = theme.meta.get("fonts").cloned().unwrap_or_default();
    let display = extract_font(&fonts_meta, "display").unwrap_or_else(|| "Inter".into());
    let body    = extract_font(&fonts_meta, "body").unwrap_or_else(|| "Inter".into());
    let mono    = extract_font(&fonts_meta, "mono").unwrap_or_else(|| "JetBrains Mono".into());

    let mut fonts_needed = vec![display.clone(), body.clone(), mono.clone()];
    fonts_needed.sort(); fonts_needed.dedup();
    let font_families: Vec<String> = fonts_needed.iter()
        .filter(|f| !f.is_empty())
        .map(|f| format!("family={}:ital,wght@0,400;0,500;0,600;0,700;1,400", f.replace(' ', "+")))
        .collect();
    let font_url = if font_families.is_empty() {
        String::new()
    } else {
        format!("https://fonts.googleapis.com/css2?{}&display=swap", font_families.join("&"))
    };

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\"><head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\n");
    html.push_str(&format!("<meta name=\"assembled\" content=\"theme:{} palette:{}\">\n", theme.id, palette.id));
    html.push_str(&format!("<title>{}</title>\n", html_escape(idea)));
    if !font_url.is_empty() {
        html.push_str("<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n");
        html.push_str("<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n");
        html.push_str(&format!("<link href=\"{}\" rel=\"stylesheet\">\n", font_url));
    }

    html.push_str("<style>\n");
    html.push_str("*,*::before,*::after{box-sizing:border-box;margin:0;padding:0;}\n");
    html.push_str(":root{\n");
    html.push_str(&palette.body); html.push('\n');
    html.push_str(&format!("--font-display: '{}', Georgia, serif;\n", display));
    html.push_str(&format!("--font-body: '{}', -apple-system, sans-serif;\n", body));
    html.push_str(&format!("--font-mono: '{}', ui-monospace, monospace;\n", mono));
    html.push_str("}\n");
    html.push_str("body{font-family:var(--font-body);color:var(--ink);background:var(--paper);line-height:1.5;-webkit-font-smoothing:antialiased;}\n");
    html.push_str("html{scroll-behavior:smooth;}\n");
    html.push_str("img{max-width:100%;display:block;}\n");
    html.push_str("a{color:inherit;}\n");
    // Section-specific styles
    for v in sections {
        html.push_str(&v.style);
        html.push('\n');
    }
    // Subtle ambient motion baked in — scroll fade-in helper (opt-in per-element).
    html.push_str("@keyframes fadeInUp{from{opacity:0;transform:translateY(16px);}to{opacity:1;transform:translateY(0);}}\n");
    html.push_str("section{animation:fadeInUp 0.6s ease-out both;}\n");
    html.push_str("@media (prefers-reduced-motion: reduce){section{animation:none;}}\n");
    html.push_str("</style>\n</head>\n<body>\n");

    // Fill placeholders in each section's HTML and append.
    for v in sections {
        let mut body_html = v.html.clone();
        for p in &v.placeholders {
            let key = format!("{{{{{}}}}}", p);
            let val = fills.get(p).cloned().unwrap_or_else(|| p.replace('_', " ").to_lowercase());
            body_html = body_html.replace(&key, &val);
        }
        html.push_str(&body_html);
        html.push('\n');
    }

    html.push_str("</body>\n</html>\n");
    html
}

/// Parse `display="Instrument Serif" body="Inter" mono="JetBrains Mono"` style
/// meta string into individual font names.
fn extract_font(meta: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let pos = meta.find(&needle)?;
    let after = &meta[pos + needle.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&#39;")
}

/// Replace the section matching `data-section` of the same category as
/// `variant_id` with the variant's HTML + styles injected. Content-preservation
/// via structural mapping: h1/h2/h3 → h1/h2/h3, first img → first img.
fn swap_section_in_html(current: &str, variant_id: &str) -> Result<String> {
    let lib = variants::library();
    let variant = lib.variants.get(variant_id)
        .ok_or_else(|| anyhow::anyhow!("unknown variant: {}", variant_id))?;

    // Find the existing section by data-section=variant.category
    let marker = format!("data-section=\"{}\"", variant.category);
    let section_start = current.find(&marker)
        .ok_or_else(|| anyhow::anyhow!("no <section data-section=\"{}\"> found", variant.category))?;

    // Walk back to find the opening `<` of this element.
    let tag_start = current[..section_start].rfind('<')
        .ok_or_else(|| anyhow::anyhow!("malformed HTML"))?;

    // Determine the tag name (nav, section, header, footer).
    let tag_end = current[tag_start..].find(|c: char| c.is_whitespace() || c == '>')
        .ok_or_else(|| anyhow::anyhow!("malformed opening tag"))?;
    let tag_name = &current[tag_start+1..tag_start+tag_end];

    // Find matching closing tag with brace-matching (nested same-name tags).
    let close = format!("</{}>", tag_name);
    let open  = format!("<{}", tag_name);
    let mut depth = 1;
    let mut idx = tag_start + tag_end;
    // Advance to just after this opening tag's `>`.
    idx = idx + current[idx..].find('>').unwrap_or(0) + 1;

    let mut section_end = idx;
    while idx < current.len() && depth > 0 {
        let next_open  = current[idx..].find(&open);
        let next_close = current[idx..].find(&close);
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => { depth += 1; idx += o + open.len(); }
            (Some(_), Some(c)) => {
                depth -= 1;
                if depth == 0 { section_end = idx + c + close.len(); break; }
                idx += c + close.len();
            }
            (None, Some(c)) => {
                depth -= 1;
                if depth == 0 { section_end = idx + c + close.len(); break; }
                idx += c + close.len();
            }
            _ => break,
        }
    }

    // Extract old content for mapping.
    let old_section = &current[tag_start..section_end];
    let mapped = map_content(old_section, &variant.html);

    // 1) Replace the section body FIRST using the current (unshifted) offsets.
    let mut new_full = String::with_capacity(current.len() + variant.html.len() + variant.style.len());
    new_full.push_str(&current[..tag_start]);
    new_full.push_str(&mapped);
    new_full.push_str(&current[section_end..]);

    // 2) THEN inject the variant's CSS into the <style> block (order matters —
    //    doing it earlier would shift offsets and corrupt the splice).
    if !variant.style.is_empty() && !new_full.contains(&variant.style) {
        if let Some(style_close) = new_full.find("</style>") {
            new_full.insert_str(style_close, &format!("\n{}\n", variant.style));
        }
    }

    Ok(new_full)
}

/// Best-effort content preservation: pull h1/h2/h3 text and first img src from
/// old section, splice into new. Placeholder tokens remaining in the new variant
/// get filled with either a matched extraction or a sensible default.
fn map_content(old: &str, new: &str) -> String {
    let old_h1 = pluck_between(old, "<h1", "</h1>").or_else(|| pluck_between(old, "<h2", "</h2>"));
    let old_sub = pluck_between(old, "<p class=\"sub", "</p>")
        .or_else(|| pluck_between(old, "<p", "</p>"));
    let old_img = pluck_attr(old, "img", "src");

    let mut out = new.to_string();

    // Replace remaining {{...}} placeholders with mapped values or defaults.
    let re = regex::Regex::new(r"\{\{([A-Z0-9_]+)\}\}").unwrap();
    let mut result = String::new();
    let mut last = 0;
    for cap in re.captures_iter(&out) {
        let m = cap.get(0).unwrap();
        let key = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        result.push_str(&out[last..m.start()]);
        let val = match key {
            k if k.starts_with("HEADLINE") || k.contains("HEADLINE") => {
                strip_tags(&old_h1.clone().unwrap_or_else(|| k.replace('_', " ").to_lowercase()))
            }
            k if k.contains("SUB") || k.contains("BODY") => {
                strip_tags(&old_sub.clone().unwrap_or_else(|| k.replace('_', " ").to_lowercase()))
            }
            k if k.contains("IMAGE") || k.contains("URL") => {
                old_img.clone().unwrap_or_else(|| "https://loremflickr.com/1200/675/design?lock=1".into())
            }
            _ => key.replace('_', " ").to_lowercase(),
        };
        result.push_str(&val);
        last = m.end();
    }
    result.push_str(&out[last..]);
    result
}

fn pluck_between(html: &str, open: &str, close: &str) -> Option<String> {
    let start = html.find(open)?;
    let after_open = &html[start..];
    let gt = after_open.find('>')?;
    let content_start = start + gt + 1;
    let end = html[content_start..].find(close)?;
    Some(html[content_start..content_start + end].to_string())
}

fn pluck_attr(html: &str, tag: &str, attr: &str) -> Option<String> {
    let needle = format!("<{}", tag);
    let start = html.find(&needle)?;
    let close = html[start..].find('>')?;
    let tag_str = &html[start..start + close];
    let attr_needle = format!("{}=\"", attr);
    let apos = tag_str.find(&attr_needle)?;
    let after = &tag_str[apos + attr_needle.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn strip_tags(s: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(s, "").trim().to_string()
}

/// Spawn a background critique. We don't block the design surface on it; the
/// LLM sends CritiqueFixes when ready. Errors are swallowed silently — the
/// critique is an assistive polish, not a hard requirement.
fn spawn_critique(html: String, provider: Provider, tx: Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = run_critique(html, provider, tx).await;
    });
}

async fn run_critique(html: String, provider: Provider, tx: Sender<AppEvent>) -> Result<()> {
    // Cap the input HTML so the critique input stays bounded.
    let bounded = bound_html(&html, 30_000);
    let system  = ai::prompts::CRITIQUE_SYSTEM;
    let user    = ai::prompts::critique_user(&bounded);

    // Non-streaming — critique is small (~500 tokens output). Routes through
    // the cached channel so the design-knowledge prefix is reused from the
    // just-completed generation (same 5-minute cache window).
    let stop = Arc::new(AtomicBool::new(false));
    let comp = provider.complete_streaming_cached(
        system, ai::prompts::SYSTEM_CONTEXT, &user, 900,
        Box::new(|_| {}),
        stop,
    ).await?;

    let items = parse_critique_json(&comp.text);
    if items.is_empty() { return Ok(()); }

    let payload = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
    let _ = tx.send(AppEvent::CritiqueFixes(payload));
    Ok(())
}

/// Parse the critique response into a Vec of {label, prompt} objects, tolerant
/// of markdown fences and surrounding prose.
fn parse_critique_json(text: &str) -> Vec<serde_json::Value> {
    let trimmed = text.trim();
    // Strip common markdown fences the LLM sometimes adds despite instructions.
    let stripped = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Find first '[' and last ']' — extract JSON substring.
    let start = stripped.find('[');
    let end   = stripped.rfind(']');
    let json_str = match (start, end) {
        (Some(a), Some(b)) if b > a => &stripped[a..=b],
        _ => return vec![],
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v, Err(_) => return vec![],
    };
    let arr = match parsed.as_array() { Some(a) => a, None => return vec![] };

    arr.iter()
        .filter(|v| v["label"].is_string() && v["prompt"].is_string())
        .take(3)
        .cloned()
        .collect()
}

fn extract_colors(html: &str) -> Vec<String> {
    let re = Regex::new(r"#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{3})\b").unwrap();
    let mut seen = std::collections::HashSet::new();
    re.find_iter(html)
        .map(|m| m.as_str().to_uppercase())
        .filter(|c| seen.insert(c.clone()))
        .take(8).collect()
}

fn extract_fonts(html: &str) -> Vec<String> {
    let re = Regex::new(r#"font-family\s*:\s*([^;}{]+)"#).unwrap();
    let mut seen = std::collections::HashSet::new();
    re.captures_iter(html)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str().split(',').next().unwrap_or("").trim()
                  .trim_matches('"').trim_matches('\'').to_string())
        .filter(|f| !f.is_empty() && seen.insert(f.clone()))
        .take(5).collect()
}
