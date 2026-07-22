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
    /// Multi-page manifest sent to UI to build the tab bar.
    PagesList { pages: String, active: String }, // pages is JSON
    /// After a design lands, suggest sub-pages the user could design next
    /// based on unwired nav links in the just-generated HTML.
    PageSuggestions { candidates: String }, // JSON: [{slug, name}]
    /// Full per-project session snapshot sent on project open / switch so the
    /// React store can rehydrate mode, brief, and token totals in one shot.
    SessionSnapshot {
        mode:       &'static str,
        brief:      String,
        tokens_in:  u64,
        tokens_out: u64,
    },
    /// User picked (or the classifier resolved) a mode. UI updates its badge.
    ModeSet { mode: &'static str },
    /// `start_design` classified the idea as Ambiguous and needs the user to
    /// pick Landing or App via the clarify picker.
    ModeClarify { brief: String },
    /// Batch skeleton generation finished — the payload is a JSON array of
    /// {slug, name, built, has_skeleton} entries so the UI can refresh its
    /// tab bar with skeleton dashes for un-built pages.
    SkeletonsReady { pages: String },
    /// Progress ping during batch skeleton generation. Lets the user see
    /// "Wireframing 2 of 4: Workouts…" instead of a silent wait.
    SkeletonProgress { current: u32, total: u32, page: String },
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
    let mut current_page:    String = "home".into();
    let mut current_mode:   crate::session::Mode = crate::session::Mode::Ambiguous;
    let mut session_input:  u64 = 0;
    let mut session_output: u64 = 0;

    // Track the last-sent model so we can re-emit Meta when the provider
    // (e.g. claudecode) discloses a different one mid-run. Seed with the
    // configured value; overridden on the first LLM completion.
    let mut current_model = model.clone();
    let send_meta = |tx: &Sender<AppEvent>, m: &str| {
        let _ = tx.send(AppEvent::Meta { provider: provider_id.clone(), model: m.to_string() });
    };

    send_meta(&tx, &current_model);
    // If the provider already knows its model (SDK providers), publish it now.
    if let Some(m) = provider.detected_model() {
        if !m.is_empty() && m != current_model {
            current_model = m.clone();
            send_meta(&tx, &current_model);
        }
    }
    push_projects_list(&tx);

    // Called after every LLM completion — cheap check, re-emits Meta only
    // when the detected model actually changed.
    let mut refresh_model = |tx: &Sender<AppEvent>, current_model: &mut String| {
        if let Some(m) = provider.detected_model() {
            if !m.is_empty() && m != *current_model {
                *current_model = m.clone();
                let _ = tx.send(AppEvent::Meta {
                    provider: provider_id.clone(),
                    model:    m,
                });
            }
        }
    };

    while let Some(msg) = rx.recv().await {
        match msg.kind.as_str() {
            "export"           => { let _ = tx.send(AppEvent::ExportDesign(msg.content));   continue; }
            "export_prototype" => { let _ = tx.send(AppEvent::ExportPrototype(msg.content)); continue; }

            "list_projects" => { send_meta(&tx, &current_model); push_projects_list(&tx); continue; }

            "create_project" => {
                // Kill any in-flight generation on the outgoing project + flush
                // its session before switching.
                stop_flag.store(true, Ordering::SeqCst);
                if let Some(ref outgoing) = current_project {
                    let brief = match &state {
                        State::Refining { idea, .. } => idea.as_str(), _ => "",
                    };
                    let tried = match &state {
                        State::Refining { tried_archetypes, .. } => tried_archetypes.clone(),
                        _ => Vec::new(),
                    };
                    persist_session(outgoing, current_mode, brief, &current_page, &tried, session_input, session_output);
                }
                stop_flag.store(false, Ordering::SeqCst);

                match projects::create(msg.content.trim()) {
                    Ok(p) => {
                        current_project = Some(p.slug.clone());
                        current_page = "home".into();
                        current_mode = crate::session::Mode::Ambiguous;
                        session_input = 0;
                        session_output = 0;
                        state = State::AwaitingIdea;
                        // Seed session on disk so subsequent opens are consistent.
                        let mut sess = crate::session::Session::default();
                        sess.mode = current_mode;
                        let _ = projects::write_session(&p.slug, &sess);

                        let _ = tx.send(AppEvent::ProjectOpened {
                            slug: p.slug.clone(),
                            name: p.name.clone(),
                            html: String::new(),
                            chat: "[]".into(),
                        });
                        emit_session_snapshot(&tx, &sess);
                        push_projects_list(&tx);
                        push_pages(&tx, &p.slug);
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

                // Kill any in-flight generation on the outgoing project + flush
                // its session before switching. This is what makes
                // per-project state safe when the user clicks another rail item
                // mid-generation.
                stop_flag.store(true, Ordering::SeqCst);
                if let Some(ref outgoing) = current_project {
                    if *outgoing != slug {
                        let brief = match &state {
                            State::Refining { idea, .. } => idea.as_str(), _ => "",
                        };
                        let tried = match &state {
                            State::Refining { tried_archetypes, .. } => tried_archetypes.clone(),
                            _ => Vec::new(),
                        };
                        persist_session(outgoing, current_mode, brief, &current_page, &tried, session_input, session_output);
                    }
                }
                stop_flag.store(false, Ordering::SeqCst);

                let manifest = projects::read_pages_manifest(&slug).unwrap_or_default();
                current_page = manifest.active.clone();
                let read_result = if current_page == "home" {
                    projects::read(&slug)
                } else {
                    projects::read_page(&slug, &current_page)
                };
                match read_result {
                    Ok(html) => {
                        current_project = Some(slug.clone());
                        let name = projects::name_of(&slug).unwrap_or_default();
                        // Prefer new jsonl chat; fall back to legacy .chat.json.
                        let chat = projects::read_chat_jsonl(&slug).ok()
                            .filter(|s| s.trim() != "[]" && !s.trim().is_empty())
                            .or_else(|| projects::read_chat(&slug).ok())
                            .unwrap_or_else(|| "[]".into());

                        // Load session, restoring mode + tokens + tried_archetypes.
                        let sess = projects::read_session(&slug).unwrap_or_default();
                        current_mode   = sess.mode;
                        session_input  = sess.tokens_in;
                        session_output = sess.tokens_out;

                        if !html.trim().is_empty() {
                            state = State::Refining {
                                current: html.clone(),
                                idea:    if sess.brief.is_empty() { name.clone() } else { sess.brief.clone() },
                                theme:   String::new(),
                                tried_archetypes: sess.tried_archetypes.clone(),
                            };
                        } else {
                            state = State::AwaitingIdea;
                        }

                        let _ = tx.send(AppEvent::ProjectOpened { slug: slug.clone(), name, html, chat });
                        emit_session_snapshot(&tx, &sess);
                        // Emit current token totals so the pill matches the project.
                        let _ = tx.send(AppEvent::TokenUsage {
                            turn_input: 0, turn_output: 0,
                            session_input, session_output, estimated: false,
                        });
                        push_pages(&tx, &slug);
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Could not open project: {}", e)
                        ));
                    }
                }
                continue;
            }

            "set_mode" => {
                let m = crate::session::Mode::from_str(msg.content.trim())
                    .unwrap_or(crate::session::Mode::Ambiguous);
                current_mode = m;
                let _ = tx.send(AppEvent::ModeSet { mode: m.as_str() });
                if let Some(ref slug) = current_project {
                    let brief = match &state {
                        State::Refining { idea, .. } => idea.as_str(), _ => "",
                    };
                    let tried = match &state {
                        State::Refining { tried_archetypes, .. } => tried_archetypes.clone(),
                        _ => Vec::new(),
                    };
                    persist_session(slug, m, brief, &current_page, &tried, session_input, session_output);
                }
                // No auto-replay of a queued start_design — the UI held the
                // payload and will re-send `start_design` once the badge flips.
                continue;
            }

            "save_chat" => {
                if let Some(ref cur_slug) = current_project {
                    // Payload shapes (backwards-compatible):
                    //   NEW: {"slug":"…","chat":[…]}  → verify slug then write
                    //   OLD: "[…]" (raw array string)  → assume current_project
                    // The NEW shape prevents a stale debounced save from an
                    // outgoing project clobbering an incoming project's chat.
                    let parsed: serde_json::Value =
                        serde_json::from_str(&msg.content).unwrap_or(serde_json::Value::Null);
                    let (target_slug, chat_json) = match parsed.get("slug").and_then(|v| v.as_str()) {
                        Some(s) => {
                            let chat = parsed.get("chat")
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "[]".into());
                            (Some(s.to_string()), chat)
                        }
                        None => (None, msg.content.clone()),
                    };
                    if let Some(ref t) = target_slug {
                        if t != cur_slug {
                            // Stale save from a project we've since switched
                            // away from — drop silently rather than corrupt.
                            continue;
                        }
                    }
                    let _ = projects::overwrite_chat_from_array(cur_slug, &chat_json);
                    let _ = projects::write_chat(cur_slug, &chat_json);
                }
                continue;
            }

            "refine_skeleton" => {
                // Chat refinement while viewing a wireframe. Rewrites the
                // page's skeleton HTML in place (never touches the built
                // version, which the user still hasn't upgraded to).
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let target = payload["slug"].as_str().unwrap_or("").trim().to_string();
                let prompt = payload["prompt"].as_str().unwrap_or("").trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                if target.is_empty() || prompt.is_empty() { continue; }
                let existing = match projects::read_skeleton(&slug, &target) {
                    Ok(s) => s, Err(_) => continue,
                };

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));
                let _ = tx.send(AppEvent::StatusUpdate(
                    format!("Tweaking {} wireframe…", target)
                ));

                let system = SKELETON_REFINE_SYSTEM;
                let user   = format!(
                    "Current wireframe HTML:\n{}\n\nUser tweak: {}\n\n\
                    Return the full updated wireframe HTML. Keep it a wireframe \
                    (grayscale, boxes, dashed placeholders) — DO NOT add real \
                    styling. No prose. No markdown fences.",
                    bound_html(&existing, 20_000), prompt
                );
                let comp = stream_generate(
                    &provider, crate::session::Mode::App,
                    system, &user, 8_000, &tx, stop_flag.clone(),
                ).await;
                let _ = tx.send(AppEvent::SetGenerating(false));
                match comp {
                    Ok(c) => {
                        let new_html = c.text.trim().to_string();
                        if !new_html.is_empty() && new_html.to_ascii_lowercase().contains("<html") {
                            let usage = usage_or_estimate(c.usage, system, &user, &new_html);
                            let _ = projects::write_skeleton(&slug, &target, &new_html);
                            emit_usage(&tx, usage, &mut session_input, &mut session_output);
                            refresh_model(&tx, &mut current_model);
                            let _ = tx.send(AppEvent::DesignUpdate(new_html));
                            let _ = tx.send(AppEvent::AssistantMessage(
                                format!("Wireframe updated. Hit \"Build this page\" when you're happy with the layout.")
                            ));
                        } else {
                            let _ = tx.send(AppEvent::AssistantMessage(
                                "Couldn't refine the wireframe — please try a more specific tweak.".into()
                            ));
                        }
                    }
                    Err(_) => {}
                }
                continue;
            }

            "get_page_skeleton" => {
                // Sends the skeleton HTML for the currently-viewed page as a
                // DesignUpdate (viewer-only — doesn't touch state). Used by
                // the Wireframe / Built toggle in the canvas toolbar.
                let target = msg.content.trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                let page = if target.is_empty() { current_page.clone() } else { target };
                if let Ok(html) = projects::read_skeleton(&slug, &page) {
                    let _ = tx.send(AppEvent::DesignUpdate(html));
                }
                continue;
            }

            "list_pages" => {
                if let Some(ref slug) = current_project { push_pages(&tx, slug); }
                continue;
            }

            "switch_page" => {
                let target = msg.content.trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                let manifest = projects::read_pages_manifest(&slug).unwrap_or_default();
                if !manifest.pages.iter().any(|p| p.slug == target) { continue; }

                // Save the current page's HTML before switching.
                persist_current(&current_project, &current_page, &state, &tx);

                current_page = target.clone();
                let _ = projects::set_active_page(&slug, &target);

                // Preference order: built page → skeleton → empty. The tab
                // bar shows the skeleton state via manifest.has_skeleton so
                // the user knows which one is being previewed.
                let html = if target == "home" {
                    projects::read(&slug).unwrap_or_default()
                } else if projects::has_built_page(&slug, &target) {
                    projects::read_page(&slug, &target).unwrap_or_default()
                } else if projects::has_skeleton(&slug, &target) {
                    projects::read_skeleton(&slug, &target).unwrap_or_default()
                } else {
                    projects::read_page(&slug, &target).unwrap_or_default()
                };
                state = if html.trim().is_empty() {
                    State::AwaitingIdea
                } else {
                    State::Refining {
                        current: html.clone(),
                        idea:    projects::name_of(&slug).unwrap_or_default(),
                        theme:   String::new(),
                        tried_archetypes: Vec::new(),
                    }
                };
                let _ = tx.send(AppEvent::DesignUpdate(html));
                push_pages(&tx, &slug);
                continue;
            }

            "build_page_from_skeleton" => {
                let target = msg.content.trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                if target.is_empty() { continue; }
                let manifest = projects::read_pages_manifest(&slug).unwrap_or_default();
                let page_info = match manifest.pages.iter().find(|p| p.slug == target) {
                    Some(p) => p.clone(), None => continue,
                };
                let skeleton_hint = projects::read_skeleton(&slug, &target).ok();
                let brief = skeleton_hint
                    .map(|s| format!(
                        "The wireframe for reference (shows structural intent):\n{}",
                        bound_html(&s, 8_000)
                    ))
                    .unwrap_or_default();

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));
                let _ = tx.send(AppEvent::StatusUpdate(
                    format!("Building {}…", page_info.name)
                ));

                let name_display = page_info.name.clone();
                let project_display = projects::name_of(&slug).unwrap_or_default();

                // Two paths:
                //  · home OR home isn't built yet → full generation from
                //    scratch using handle_start_design with the skeleton as
                //    brief hint. This establishes the design system.
                //  · sub-page after home built → handle_new_page inherits
                //    the home shell for consistency.
                let idea_for_build = if brief.is_empty() {
                    project_display.clone()
                } else {
                    format!("{project_display}\n\n{brief}")
                };
                let home_exists = projects::has_built_page(&slug, "home");
                let result = if target == "home" || !home_exists {
                    handle_start_design(
                        &idea_for_build, "auto", &[], &[], crate::session::Mode::App,
                        &provider, &tx, stop_flag.clone(), &knowledge,
                    ).await.map(|(state, usage)| {
                        let html = match state {
                            State::Refining { current, .. } => current,
                            _ => String::new(),
                        };
                        (html, usage)
                    })
                } else {
                    let home_html = projects::read(&slug).unwrap_or_default();
                    handle_new_page(
                        &project_display, &name_display, &brief, &home_html,
                        &provider, &tx, stop_flag.clone(),
                    ).await
                };

                let _ = tx.send(AppEvent::SetGenerating(false));
                match result {
                    Ok((built_html, usage)) => {
                        // home lands at {slug}.html; sub-pages at {slug}--{page}.html
                        if target == "home" {
                            let _ = projects::write(&slug, &built_html);
                        } else {
                            let _ = projects::write_page(&slug, &target, &built_html);
                        }
                        // Mark this page as built in the manifest.
                        if let Ok(mut m) = projects::read_pages_manifest(&slug) {
                            if let Some(p) = m.pages.iter_mut().find(|p| p.slug == target) {
                                p.built = true;
                            }
                            m.active = target.clone();
                            let _ = projects::write_pages_manifest(&slug, &m);
                        }
                        current_page = target.clone();
                        state = State::Refining {
                            current: built_html.clone(),
                            idea: project_display.clone(),
                            theme: String::new(),
                            tried_archetypes: Vec::new(),
                        };
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        refresh_model(&tx, &mut current_model);
                        let _ = tx.send(AppEvent::DesignUpdate(built_html));
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Built the {name_display} page. Toggle to the wireframe from the canvas toolbar anytime.")
                        ));
                        push_pages(&tx, &slug);

                        // After building, offer the remaining wireframes as
                        // chips so the user picks the next one to build.
                        if let Ok(m) = projects::read_pages_manifest(&slug) {
                            let remaining: Vec<serde_json::Value> = m.pages.iter()
                                .filter(|p| p.slug != "home" && !p.built && p.has_skeleton)
                                .map(|p| serde_json::json!({ "slug": p.slug, "name": p.name }))
                                .collect();
                            if !remaining.is_empty() {
                                let json = serde_json::to_string(&remaining).unwrap_or_else(|_| "[]".into());
                                let _ = tx.send(AppEvent::PageSuggestions { candidates: json });
                            }
                        }
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Couldn't build {name_display}: {e}")
                        ));
                    }
                }
                continue;
            }

            "delete_page" => {
                let target = msg.content.trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                let _ = projects::delete_page(&slug, &target);
                if current_page == target { current_page = "home".into(); }
                push_pages(&tx, &slug);
                // Reload the (now active) home page into the canvas.
                let html = projects::read(&slug).unwrap_or_default();
                let _ = tx.send(AppEvent::DesignUpdate(html));
                continue;
            }

            "create_page" => {
                let payload: serde_json::Value = serde_json::from_str(&msg.content)
                    .unwrap_or(serde_json::Value::Null);
                let name = payload["name"].as_str().unwrap_or("").trim().to_string();
                let brief = payload["brief"].as_str().unwrap_or("").trim().to_string();
                let slug = match &current_project { Some(s) => s.clone(), None => continue };
                if name.is_empty() { continue; }

                // Load home page HTML — it's the shell donor.
                let home_html = projects::read(&slug).unwrap_or_default();
                if home_html.trim().is_empty() {
                    let _ = tx.send(AppEvent::AssistantMessage(
                        "Design the Home page first — new pages inherit its shell.".into()
                    ));
                    continue;
                }

                // Reserve the page slot in the manifest and make it active.
                let page_slug = match projects::add_page(&slug, &name) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(format!("Couldn't add page: {}", e)));
                        continue;
                    }
                };
                current_page = page_slug.clone();
                push_pages(&tx, &slug);

                let project_name = projects::name_of(&slug).unwrap_or_default();
                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));
                let _ = tx.send(AppEvent::StatusUpdate(
                    format!("Designing the {} page…", name)
                ));

                let result = handle_new_page(
                    &project_name, &name, &brief, &home_html,
                    &provider, &tx, stop_flag.clone(),
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((html, usage)) => {
                        state = State::Refining {
                            current: html.clone(),
                            idea:    project_name.clone(),
                            theme:   String::new(),
                            tried_archetypes: Vec::new(),
                        };
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        refresh_model(&tx, &mut current_model);
                        persist_current(&current_project, &current_page, &state, &tx);
                        let _ = tx.send(AppEvent::DesignUpdate(html));
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Ready — {} page designed. Switch pages from the tabs above the canvas.", name)
                        ));
                    }
                    Err(e) if e.to_string() == "__stopped__" => {}
                    Err(e) => {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            format!("Couldn't design the {} page: {}", name, e)
                        ));
                    }
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
                let initial_pages: Vec<String> = payload["initial_pages"].as_array()
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect())
                    .unwrap_or_default();
                if idea.is_empty() { continue; }

                // Mode routing: honor an explicitly-set mode on the project,
                // else classify from the idea. Ambiguous → emit clarify chip
                // and stop — the UI re-sends start_design once user picks.
                if current_mode == crate::session::Mode::Ambiguous {
                    let inferred = crate::session::infer_mode(&idea);
                    if inferred == crate::session::Mode::Ambiguous {
                        let _ = tx.send(AppEvent::ModeClarify { brief: idea.clone() });
                        continue;
                    }
                    current_mode = inferred;
                    let _ = tx.send(AppEvent::ModeSet { mode: current_mode.as_str() });
                    if let Some(ref slug) = current_project {
                        persist_session(slug, current_mode, &idea, &current_page, &[], session_input, session_output);
                    }
                }

                stop_flag.store(false, Ordering::SeqCst);
                let _ = tx.send(AppEvent::SetGenerating(true));

                // ── APP mode: skeleton-first fork ─────────────────────────
                // Skip the full-fidelity home build. Generate wireframes for
                // EVERY page (home + declared/inferred siblings) so the user
                // can review the whole app's structure, tweak any wireframe
                // via chat, and pick which one to upgrade first.
                if current_mode == crate::session::Mode::App {
                    let slug = current_project.clone();
                    if slug.is_none() {
                        let _ = tx.send(AppEvent::AssistantMessage(
                            "Create a project first, then describe the app.".into()
                        ));
                        let _ = tx.send(AppEvent::SetGenerating(false));
                        continue;
                    }
                    let slug = slug.unwrap();

                    let declared = initial_pages.clone();
                    let names = if !declared.is_empty() {
                        declared
                    } else {
                        // Infer 4-5 typical pages for this idea. No home HTML
                        // exists yet — the LLM works from the brief alone.
                        suggest_pages_from_brief(&provider, &idea).await
                    };

                    let result = handle_app_skeleton_phase(
                        &slug, &idea, &names,
                        &provider, &tx, stop_flag.clone(),
                    ).await;
                    let _ = tx.send(AppEvent::SetGenerating(false));

                    match result {
                        Ok((home_skeleton, usage)) => {
                            emit_usage(&tx, usage, &mut session_input, &mut session_output);
                            refresh_model(&tx, &mut current_model);
                            persist_session(&slug, current_mode, &idea, "home", &[], session_input, session_output);
                            state = State::Refining {
                                current: home_skeleton.clone(),
                                idea:    idea.clone(),
                                theme:   String::new(),
                                tried_archetypes: Vec::new(),
                            };
                            current_page = "home".to_string();
                            let _ = tx.send(AppEvent::DesignUpdate(home_skeleton));
                            let _ = tx.send(AppEvent::AssistantMessage(
                                "Wireframed the whole app. Click any tab to preview — tweak the wireframe via chat, or hit \"Build this page\" to upgrade to full fidelity.".into()
                            ));
                        }
                        Err(e) if e.to_string() == "__stopped__" => {}
                        Err(e) => {
                            let _ = tx.send(AppEvent::AssistantMessage(
                                format!("Couldn't wireframe the app: {}\n\nTry again.", e)
                            ));
                        }
                    }
                    continue;
                }

                let result = handle_start_design(
                    &idea, &theme, &[], &initial_pages, current_mode,
                    &provider, &tx, stop_flag.clone(), &knowledge,
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        refresh_model(&tx, &mut current_model);
                        persist_current(&current_project, &current_page, &state, &tx);
                        // Persist the session with the fresh brief + tokens.
                        if let Some(ref slug) = current_project {
                            let tried = match &state {
                                State::Refining { tried_archetypes, .. } => tried_archetypes.clone(),
                                _ => Vec::new(),
                            };
                            persist_session(slug, current_mode, &idea, &current_page, &tried, session_input, session_output);
                        }
                        // On the home page of a project, suggest linked sub-pages
                        // the user could design next based on the generated nav.
                        if current_page == "home" {
                            if let (Some(slug), State::Refining { current, .. }) = (current_project.as_ref(), &state) {
                                push_pages(&tx, slug);
                                push_page_suggestions(&tx, &provider, &idea, slug, current);
                            }
                        }
                        if let State::Refining { current, .. } = &state {
                            spawn_critique(current.clone(), current_mode, provider.clone(), tx.clone());
                        }

                        // Note: APP-mode skeleton generation runs BEFORE
                        // reaching this point, via the skeleton-first fork
                        // above. This success branch only handles LANDING
                        // mode now, so no auto-skeleton spawn here.
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
                        refresh_model(&tx, &mut current_model);
                        persist_current(&current_project, &current_page, &state, &tx);
                        // Kit-picker path = landing pages. Single-page by design —
                        // no "design next page?" chips.
                        if current_page == "home" {
                            if let Some(slug) = current_project.as_ref() {
                                push_pages(&tx, slug);
                            }
                        }
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
                        persist_current(&current_project, &current_page, &state, &tx);
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
                    &idea, &theme, &tried, &[], current_mode,
                    &provider, &tx, stop_flag.clone(), &knowledge,
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        refresh_model(&tx, &mut current_model);
                        persist_current(&current_project, &current_page, &state, &tx);
                        if let State::Refining { current, .. } = &state {
                            spawn_critique(current.clone(), current_mode, provider.clone(), tx.clone());
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
                    current_mode, &provider, &tx, stop_flag.clone(),
                ).await;

                let _ = tx.send(AppEvent::SetGenerating(false));

                match result {
                    Ok((next, usage)) => {
                        state = next;
                        emit_usage(&tx, usage, &mut session_input, &mut session_output);
                        refresh_model(&tx, &mut current_model);
                        persist_current(&current_project, &current_page, &state, &tx);
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
                    &content, current, idea, theme, tried_archetypes, current_mode,
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
                        refresh_model(&tx, &mut current_model);
                persist_current(&current_project, &current_page, &state, &tx);
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

/// Design a sub-page of an existing project. The home page's HTML is the
/// "shell donor" — the nav / sidebar / topbar / theme must survive unchanged.
/// Only the main workspace changes to be the {page_name} content.
async fn handle_new_page(
    project_name: &str,
    page_name:    &str,
    user_brief:   &str,
    home_html:    &str,
    provider:     &Provider,
    tx:           &Sender<AppEvent>,
    stop:         Arc<AtomicBool>,
) -> Result<(String, ai::Usage)> {
    let bounded_home = bound_html(home_html, 30_000);

    let system = "You are a senior UI/UX designer designing an additional page \
of a multi-page product UI. You will receive the HOME page's HTML as reference. \
Your output is a NEW page that lives in the same project.\n\n\
HARD RULES:\n\
- Keep the app shell IDENTICAL to the home page: <head>, <style>, all CSS \
custom properties (--paper, --ink, --accent, etc.), the sidebar/topbar/nav HTML, \
and the fonts. The user must feel they never left the app.\n\
- Update the active nav item so the {NEW_PAGE_NAME} item is marked active \
(add class=\"active\" or aria-current=\"page\") and the home item is not.\n\
- Only change the MAIN WORKSPACE (the primary content area — everything that \
isn't the sidebar/topbar/nav/footer).\n\
- Realistic domain content specific to the page's purpose (settings → account/notifications/integrations/billing; users → list + filters + detail drawer; reports → chart + table; etc).\n\
- Real inline data — never Lorem Ipsum, never round-number stats.\n\
- Output the FULL HTML document. Not a diff. Not a fragment. Not markdown fences.\n\
- Every nav link continues to point to real filenames (`./home.html`, `./{other-pages}.html`).\n\n\
Output ONLY raw HTML.";

    let user = format!(
        "Project: {project_name}\n\
New page to design: {page_name}\n\
{brief_line}\n\n\
=== HOME PAGE HTML (shell donor — reuse its head, styles, and nav exactly) ===\n\
{bounded_home}\n\n\
Now output the complete HTML for the {page_name} page. Same shell, new workspace.",
        brief_line = if user_brief.is_empty() { String::new() } else { format!("Brief: {user_brief}") },
    );

    // Sub-page of a multi-page project → always App mode context.
    let comp = stream_generate(provider, crate::session::Mode::App, system, &user, 8000, tx, stop).await?;
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);
    Ok((comp.text, usage))
}

/// Snapshot mode/tokens/tried_archetypes/brief into the project's session
/// file. Best-effort; failures are silent (session is a nice-to-have, not
/// load-bearing for the current design pass).
fn persist_session(
    slug:          &str,
    mode:          crate::session::Mode,
    brief:         &str,
    active_page:   &str,
    tried:         &[String],
    tokens_in:     u64,
    tokens_out:    u64,
) {
    let mut sess = projects::read_session(slug).unwrap_or_default();
    sess.mode = mode;
    if !brief.is_empty() { sess.brief = brief.to_string(); }
    sess.active_page = active_page.to_string();
    sess.tried_archetypes = tried.to_vec();
    sess.tokens_in  = tokens_in;
    sess.tokens_out = tokens_out;
    sess.touch();
    let _ = projects::write_session(slug, &sess);
}

fn emit_session_snapshot(tx: &Sender<AppEvent>, sess: &crate::session::Session) {
    let _ = tx.send(AppEvent::SessionSnapshot {
        mode:       sess.mode.as_str(),
        brief:      sess.brief.clone(),
        tokens_in:  sess.tokens_in,
        tokens_out: sess.tokens_out,
    });
    let _ = tx.send(AppEvent::ModeSet { mode: sess.mode.as_str() });
}

fn persist_current(
    current_project: &Option<String>,
    current_page:    &str,
    state:           &State,
    tx:              &Sender<AppEvent>,
) {
    if let (Some(slug), State::Refining { current, .. }) = (current_project.as_ref(), state) {
        // Write to the active page's file. Falls back to legacy write() for the
        // home page so existing single-file projects keep working.
        if current_page == "home" {
            let _ = projects::write(slug, current);
        } else {
            let _ = projects::write_page(slug, current_page, current);
        }
        push_projects_list(tx);
    }
}

/// After a design lands, produce up to 6 sibling-page candidates for the UI
/// "Design next?" chips. Sources, in priority order:
///
/// 1. Anchors with slug-shaped hrefs (`./workouts.html`, `/workouts`, `#workouts`).
/// 2. Labels of anchors/buttons inside nav regions (`<aside>`, `<nav>`, or
///    id/class containing sidebar/topbar/nav/menu) — captures placeholder
///    `href="#"` sidebars.
///
/// Filters: skip in-page anchors whose slug matches an existing `id="…"`,
/// skip pages already in the manifest, dedupe by slug, cap at 6.
fn suggest_next_pages(html: &str, project_slug: &str) -> Vec<(String, String)> {
    use regex::Regex;
    let anchor_re  = Regex::new(r#"<a[^>]+href=["']([^"']+)["'][^>]*>([^<]{1,60})</a>"#).unwrap();
    let id_re      = Regex::new(r#"\bid=["']([a-zA-Z0-9\-_]+)["']"#).unwrap();
    // Locate nav-scoped regions (aside / nav elements, or containers whose
    // id/class contains sidebar/topbar/nav/menu). We slice their inner HTML
    // and extract every <a>/<button> label — hrefs may be `#` placeholders.
    let nav_region_re = Regex::new(
        r#"(?is)<(aside|nav)\b[^>]*>(.*?)</\1>|<(?:div|section|header|ul)\b[^>]*(?:id|class)=["'][^"']*(?:sidebar|topbar|nav|menu)[^"']*["'][^>]*>(.*?)</(?:div|section|header|ul)>"#,
    ).unwrap();
    let label_re = Regex::new(r#"(?is)<(?:a|button)\b[^>]*>(.*?)</(?:a|button)>"#).unwrap();
    let tag_strip_re = Regex::new(r"(?is)<[^>]+>").unwrap();
    let ws_re = Regex::new(r"\s+").unwrap();

    let existing_ids: std::collections::HashSet<String> = id_re
        .captures_iter(html)
        .filter_map(|c| c.get(1).map(|m| projects::slugify(m.as_str())))
        .collect();

    let manifest = projects::read_pages_manifest(project_slug).unwrap_or_default();
    let existing_pages: std::collections::HashSet<String> =
        manifest.pages.iter().map(|p| p.slug.clone()).collect();

    let mut seen = std::collections::HashSet::new();
    let mut out  = Vec::new();

    // Helper closures.
    let is_valid_slug = |slug: &str| -> bool {
        !slug.is_empty() && slug != "home" && slug != "index"
            && !existing_pages.contains(slug) && !existing_ids.contains(slug)
    };
    let mut push_candidate = |slug: String, label: String, seen: &mut std::collections::HashSet<String>, out: &mut Vec<(String, String)>| {
        if !is_valid_slug(&slug) { return; }
        if !seen.insert(slug.clone()) { return; }
        out.push((slug, label));
    };

    // Pass 1 — hrefs with slug-shaped targets.
    for cap in anchor_re.captures_iter(html) {
        if out.len() >= 6 { break; }
        let href      = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
        let raw_label = cap.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        if href.is_empty() || raw_label.is_empty() { continue; }

        let lower = href.to_ascii_lowercase();
        if lower.starts_with("http://") || lower.starts_with("https://")
            || lower.starts_with("mailto:") || lower.starts_with("tel:")
            || lower.starts_with("javascript:") || lower.starts_with("data:")
            || lower == "#"
        { continue; }

        let mut s = href.trim_start_matches('#')
                        .trim_start_matches("./")
                        .trim_start_matches('/');
        if let Some(i) = s.find(&['?', '#'][..]) { s = &s[..i]; }
        let s = s.trim_end_matches(".html").trim_end_matches(".htm");
        if s.is_empty() { continue; }
        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/') {
            continue;
        }
        let seg = s.rsplit('/').next().unwrap_or(s);
        if seg.is_empty() { continue; }

        let slug = projects::slugify(seg);
        push_candidate(slug, raw_label.to_string(), &mut seen, &mut out);
    }

    // Pass 2 — labels inside nav-scoped regions. Only runs if pass 1 left room.
    if out.len() < 6 {
        for region_cap in nav_region_re.captures_iter(html) {
            if out.len() >= 6 { break; }
            let inner = region_cap.get(2).or_else(|| region_cap.get(3))
                .map(|m| m.as_str()).unwrap_or("");
            for lcap in label_re.captures_iter(inner) {
                if out.len() >= 6 { break; }
                let raw = lcap.get(1).map(|m| m.as_str()).unwrap_or("");
                let stripped = tag_strip_re.replace_all(raw, "");
                let label = ws_re.replace_all(stripped.trim(), " ").to_string();
                if label.is_empty() || label.len() > 60 { continue; }
                let slug = projects::slugify(&label);
                push_candidate(slug, label, &mut seen, &mut out);
            }
        }
    }

    out
}

/// Best-effort LLM fallback for `push_page_suggestions` when both href
/// extraction and nav-label extraction return no candidates. Small
/// non-streaming call; any parse error returns an empty list so the UI just
/// silently omits chips.
async fn suggest_next_pages_llm(
    provider: &Provider,
    idea: &str,
    html: &str,
    project_slug: &str,
) -> Vec<(String, String)> {
    use regex::Regex;
    let id_re = Regex::new(r#"\bid=["']([a-zA-Z0-9\-_]+)["']"#).unwrap();
    let existing_ids: Vec<String> = id_re
        .captures_iter(html)
        .filter_map(|c| c.get(1).map(|m| projects::slugify(m.as_str())))
        .collect();

    let system = ai::prompts::NEXT_PAGES_SUGGEST_SYSTEM;
    let user   = ai::prompts::next_pages_suggest_user(idea, &existing_ids);

    let comp = match provider.complete(system, &user, 200).await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let cleaned = ai::clean(comp.text);

    // Trim optional markdown fences the model might emit despite instructions.
    let json_str = cleaned.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    #[derive(serde::Deserialize)]
    struct Suggestion { name: String, slug: String }
    let parsed: Vec<Suggestion> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let manifest = projects::read_pages_manifest(project_slug).unwrap_or_default();
    let existing_pages: std::collections::HashSet<String> =
        manifest.pages.iter().map(|p| p.slug.clone()).collect();
    let existing_ids_set: std::collections::HashSet<String> =
        existing_ids.into_iter().collect();

    let mut seen = std::collections::HashSet::new();
    parsed.into_iter()
        .filter_map(|s| {
            let slug = projects::slugify(&s.slug);
            let name = s.name.trim().to_string();
            if slug.is_empty() || name.is_empty() { return None; }
            if slug == "home" || slug == "index" { return None; }
            if existing_pages.contains(&slug) { return None; }
            if existing_ids_set.contains(&slug) { return None; }
            if !seen.insert(slug.clone()) { return None; }
            Some((slug, name))
        })
        .take(4)
        .collect()
}

fn push_pages(tx: &Sender<AppEvent>, project_slug: &str) {
    let manifest = projects::read_pages_manifest(project_slug).unwrap_or_default();
    let json = serde_json::to_string(&manifest.pages).unwrap_or_else(|_| "[]".into());
    let _ = tx.send(AppEvent::PagesList { pages: json, active: manifest.active });
}

/// Fire-and-forget page-suggestion pass. Runs the fast regex extractors
/// first; if they turn up nothing, spawns a best-effort LLM fallback so the
/// UI still gets chips for dashboards whose sidebars have no navigable
/// hrefs or labels. Always emits at most one PageSuggestions event.
fn push_page_suggestions(
    tx: &Sender<AppEvent>,
    provider: &Provider,
    idea: &str,
    project_slug: &str,
    html: &str,
) {
    let candidates = suggest_next_pages(html, project_slug);
    if !candidates.is_empty() {
        emit_page_suggestions(tx, &candidates);
        return;
    }
    // No regex hits — try the LLM fallback in the background. If it also
    // returns empty, the UI simply doesn't render chips (matches today's
    // behavior). Never blocks the main design flow.
    let tx2       = tx.clone();
    let provider2 = provider.clone();
    let idea2     = idea.to_string();
    let slug2     = project_slug.to_string();
    let html2     = html.to_string();
    tokio::spawn(async move {
        let candidates = suggest_next_pages_llm(&provider2, &idea2, &html2, &slug2).await;
        if !candidates.is_empty() {
            emit_page_suggestions(&tx2, &candidates);
        }
    });
}

fn emit_page_suggestions(tx: &Sender<AppEvent>, candidates: &[(String, String)]) {
    let json: String = serde_json::to_string(
        &candidates.iter()
            .map(|(slug, name)| serde_json::json!({ "slug": slug, "name": name }))
            .collect::<Vec<_>>()
    ).unwrap_or_else(|_| "[]".into());
    let _ = tx.send(AppEvent::PageSuggestions { candidates: json });
}

fn push_projects_list(tx: &Sender<AppEvent>) {
    let items = projects::list().unwrap_or_default();
    let json = serde_json::to_string(&items).unwrap_or_else(|_| "[]".into());
    let _ = tx.send(AppEvent::ProjectsList(json));
}

/// System prompt for tweaking an existing wireframe. Kept adjacent to the
/// handler so the whole feature is easy to trace.
const SKELETON_REFINE_SYSTEM: &str = "\
You are refining an existing wireframe HTML document. Apply the user's tweak \
and return the FULL updated document. \n\n\
Rules:\n\
- KEEP the wireframe aesthetic: grayscale only, 1px solid gray borders, \
dashed placeholders for media/images (`class=\"wf-box dashed\"`), `.wireframe` \
wrapper class, `color:#555`, no color, no shadows, no gradients.\n\
- Preserve the home page shell (sidebar, topbar, nav, footer) so the wireframe \
still feels like part of the same app.\n\
- The user's tweak targets the workspace region — add/remove/reorder wireframe \
boxes as requested, but don't upgrade any part to full fidelity.\n\
- Preserve the `<meta name=\"page-mode\" content=\"skeleton\">` tag.\n\
Output ONLY the raw HTML. No prose. No markdown fences.";

async fn stream_generate(
    provider: &Provider, mode: crate::session::Mode,
    system: &str, user: &str, max_tokens: u32,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
) -> Result<ai::Completion> {
    let tx2 = tx.clone();
    // The mode-scoped system context is a big static prefix identical across
    // every call within a mode. Sending it as a cacheable block enables
    // Anthropic prompt caching and OpenAI's implicit caching.
    let comp = provider.complete_streaming_cached(
        system, ai::prompts::system_context(mode), user, max_tokens,
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
    initial_pages: &[String],
    mode: crate::session::Mode,
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

    // Scrape design references (Awwwards, SiteInspire) with a hard 8s timeout
    // so a slow external site can't block the whole design flow. If it fails or
    // times out we proceed with an empty refs block — the design knowledge base
    // in SYSTEM_CONTEXT is enough to produce a good design on its own.
    let _ = tx.send(AppEvent::StatusUpdate("Scraping design references...".into()));
    let refs = match tokio::time::timeout(
        std::time::Duration::from_secs(8),
        scraper::gather(idea),
    ).await {
        Ok(s) => s,
        Err(_) => String::new(),
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
    let user   = ai::prompts::skeleton_single_styled_user(idea, theme, &excluded, &combined_refs, &knowledge.prompt_context(), initial_pages);
    let comp = stream_generate(provider, mode, system, &user, 8000, tx, stop.clone()).await?;
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let archetype = extract_archetype(&comp.text).unwrap_or_else(|| "unknown".into());
    let mut all_tried = tried.to_vec();
    if !all_tried.contains(&archetype) { all_tried.push(archetype.clone()); }

    // ── Orchestrator: auto-quality pass ───────────────────────────────
    // Run the Critic → Refiner loop BEFORE the DesignUpdate lands, so the
    // user sees the polished version, not the raw one. Any leftover
    // critique items surface as clickable "polish" chips.
    let octx = crate::orchestrator::OrchestratorCtx {
        brief:    idea,
        mode,
        provider,
        tx,
        stop:     stop.clone(),
    };
    let (final_html, remaining) =
        crate::orchestrator::auto_quality_pass(&octx, comp.text, 1).await;

    let _ = tx.send(AppEvent::DesignUpdate(final_html.clone()));
    let _ = tx.send(AppEvent::AssistantMessage(format!(
        "Design ready — {} layout. Edit directly on the canvas, refine here, or ask for a different layout.",
        archetype
    )));
    // Publish any not-auto-applied critic items as polish chips.
    if !remaining.is_empty() {
        let json = serde_json::to_string(&remaining).unwrap_or_else(|_| "[]".into());
        let _ = tx.send(AppEvent::CritiqueFixes(json));
    }

    Ok((
        State::Refining {
            current: final_html,
            idea:    idea.to_string(),
            theme:   theme.to_string(),
            tried_archetypes: all_tried,
        },
        usage,
    ))
}

async fn handle_refine(
    feedback: &str, current: &str, idea: &str, theme: &str, tried: &[String],
    mode: crate::session::Mode,
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
    knowledge: &KnowledgeBase,
) -> Result<(State, ai::Usage)> {
    let _ = tx.send(AppEvent::StatusUpdate("Refining design...".into()));
    let system  = ai::prompts::REFINE_SYSTEM;
    let bounded = bound_html(current, 40_000);
    let user    = ai::prompts::refine_user(&bounded, feedback, &knowledge.prompt_context());
    let comp = stream_generate(provider, mode, system, &user, 6000, tx, stop).await?;
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
    mode: crate::session::Mode,
    provider: &Provider,
    tx: &Sender<AppEvent>, stop: Arc<AtomicBool>,
) -> Result<(State, ai::Usage)> {
    let _ = tx.send(AppEvent::StatusUpdate(
        format!("Refining {}...", if selector.is_empty() { "element" } else { selector })
    ));
    let system = ai::prompts::ELEMENT_REFINE_SYSTEM;
    let user   = ai::prompts::element_refine_user(selector, outer_html, feedback);
    let comp   = stream_generate(provider, mode, system, &user, 3000, tx, stop).await?;
    let usage  = usage_or_estimate(comp.usage, system, &user, &comp.text);

    let new_element = comp.text.trim().to_string();
    if new_element.is_empty() {
        return Err(anyhow::anyhow!("empty replacement returned"));
    }

    // Guard against the LLM returning a full HTML document (happens when the
    // user's request is broad like "change the design"). If we see doctype
    // or html/body tags at the top level, try to extract the intended
    // element by matching the outer_html's opening tag; if we can't find a
    // clean match, refuse to splice — full-page rewrite via element-scoped
    // edit is exactly the bug we're preventing.
    let new_element = sanitize_element_replacement(&new_element, outer_html)?;

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

/// The element-refine LLM sometimes returns a full HTML document when the
/// user's prompt is broad. Detect that case and either extract the intended
/// element by matching the anchor's opening tag, or bail with a clear error
/// so we don't accidentally splice `<!DOCTYPE …><html>…</html>` into the
/// middle of the page.
fn sanitize_element_replacement(raw: &str, anchor_outer: &str) -> Result<String> {
    let trimmed = raw.trim();
    let lower   = trimmed.to_ascii_lowercase();
    let looks_full_doc =
        lower.starts_with("<!doctype")
        || lower.contains("<html")
        || lower.contains("<body")
        || lower.contains("<head");
    if !looks_full_doc { return Ok(trimmed.to_string()); }

    // Try to extract the intended element by matching the anchor's opening
    // tag inside the returned document. E.g. anchor starts with
    // `<section id="hero"` → find the first `<section` in raw and grab its
    // outer HTML by tag-depth counting.
    let anchor_tag = anchor_opening_tag(anchor_outer)
        .ok_or_else(|| anyhow::anyhow!("element edit failed: couldn't parse anchor tag"))?;

    if let Some(slice) = extract_first_element(trimmed, &anchor_tag) {
        return Ok(slice);
    }

    // Couldn't recover — refuse to splice so we don't corrupt the page.
    Err(anyhow::anyhow!(
        "element edit refused: model returned a full HTML document instead of a \
         single element. Try a more specific instruction like 'make the button \
         teal' rather than 'change the design'."
    ))
}

/// Read the opening tag name from an outer HTML string. `<section id="hero">…</section>` → `section`.
fn anchor_opening_tag(outer: &str) -> Option<String> {
    let s = outer.trim_start();
    if !s.starts_with('<') { return None; }
    let end = s[1..].find(|c: char| !c.is_ascii_alphanumeric() && c != '-')?;
    let tag = &s[1..1+end];
    if tag.is_empty() { None } else { Some(tag.to_ascii_lowercase()) }
}

/// Find the first `<tag …>…</tag>` in `haystack`, matching balanced open/close
/// counts so nested same-tag children don't fool the extraction.
fn extract_first_element(haystack: &str, tag: &str) -> Option<String> {
    let open_pat  = format!("<{}", tag);
    let close_pat = format!("</{}", tag);
    let lower = haystack.to_ascii_lowercase();
    // Locate the outermost start.
    let start = lower.find(&open_pat)?;
    // Confirm it's really an opening tag boundary (not `<sectionX` etc.).
    let after = haystack.as_bytes().get(start + open_pat.len()).copied();
    if let Some(c) = after {
        if c.is_ascii_alphanumeric() { return None; }
    }
    // Walk forward, counting depth.
    let mut depth: i32 = 0;
    let mut cursor = start;
    loop {
        let rest = &lower[cursor..];
        let next_open  = rest.find(&open_pat).map(|i| cursor + i);
        let next_close = rest.find(&close_pat).map(|i| cursor + i);
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                cursor = o + open_pat.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    // find the '>' that closes this close-tag
                    let close_end = haystack[c..].find('>')? + c + 1;
                    return Some(haystack[start..close_end].to_string());
                }
                cursor = c + close_pat.len();
            }
            (Some(o), None) => {
                depth += 1;
                cursor = o + open_pat.len();
            }
            _ => return None,
        }
    }
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
    let base_palette = lib.palettes.get(&palette_key)
        .or_else(|| lib.palettes.get("warm-cream-brick"))
        .ok_or_else(|| anyhow::anyhow!("no palettes available"))?;

    // ── Scrape design references for palette uniqueness ────────────────
    // Kit-picker layouts are hand-authored (17 fixed variants); without any
    // web input, every page ends up looking the same. Blend scraped colors
    // into the chosen palette so the color story is unique per generation.
    // 8s hard timeout; failure is silent (falls back to the base palette).
    let _ = tx.send(AppEvent::StatusUpdate("Scraping refs for palette uniqueness…".into()));
    let scraped_refs = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        scraper::gather_refs(idea),
    ).await.unwrap_or_default();

    let scraped_colors: Vec<String> = scraped_refs.iter().flat_map(|r| r.colors.clone()).collect();
    let blended = blend_palette(base_palette, &scraped_colors);
    // From here on, `palette` is the blended version (or a clone of the base
    // if scraping failed or returned fewer than 3 colors).
    let palette: &crate::variants::Palette = if scraped_colors.len() >= 3 { &blended } else { base_palette };
    if scraped_colors.len() >= 3 {
        let _ = tx.send(AppEvent::StatusUpdate(
            format!("Blended {} scraped colors into palette…", scraped_colors.len())
        ));
    }

    // Build a compact ref summary for the placeholder LLM — grounds copy
    // in the tone of real award-winning sites in the subject's category.
    let refs_prompt_block = if scraped_refs.is_empty() {
        String::new()
    } else {
        let block = scraped_refs.iter().take(3).map(|r| r.summary()).collect::<Vec<_>>().join("\n");
        format!("\n\nReference sites in this category (for tone only — do NOT copy content):\n{block}\n")
    };

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

    let system = "You are a UX copywriter AND design director filling both COPY placeholders and STYLE tweaks for a web landing page. You have three inputs: the subject, the user's picked kit (theme + variants), and scraped reference sites in the same category. Blend them.\n\n\
Output STRICTLY a single JSON object with TWO top-level keys:\n\
  {\n\
    \"placeholders\": { \"KEY\": \"value\", ... },\n\
    \"tweaks\":       { \"radius\": \"8px\", \"shadow\": \"...\", \"section_gap\": \"96px\" }\n\
  }\n\n\
PLACEHOLDER rules:\n\
- Use specific nouns from the subject's world, never generic benefit words.\n\
- Headlines: 8-14 words, ONE concrete noun the audience recognizes.\n\
- Subheads: 12-24 words, elaborate the specific benefit.\n\
- CTAs: 2-4 words, verb-noun, specific first action (\"Book 20 min\", \"Import invoices\"), NEVER \"Get started\".\n\
- Stats: specific odd numbers, not round (\"4 hours saved per week — median\"), never \"10x\" or \"99.9%\".\n\
- Testimonials: named person + realistic role + realistic company + specific outcome number.\n\
- Image URLs: use https://loremflickr.com/{W}/{H}/{keywords}?lock={seed} matched to subject, OR https://i.pravatar.cc/{size}?img={1..70} for avatars.\n\
- Feature titles: 2-4 words, concrete product noun.\n\
- Use realistic email placeholders (e.g. \"you@studio.co\").\n\
- Copy tone should feel drawn from the reference sites — not copied, but same register.\n\n\
TWEAK rules (each tweak is short — either a CSS value or a name from a fixed vocabulary):\n\
- radius: corner rounding shared across cards/buttons. Pick between \"2px\" (brutalist/technical), \"4-6px\" (editorial/minimal), \"10-16px\" (playful/warm), \"999px\" (soft). Match the vibe of the theme AND the scraped refs.\n\
- shadow: box-shadow for elevated cards. Should feel consistent with the palette — cool palettes get bluish shadows, warm palettes get warm-brown shadows.\n\
- section_gap: vertical padding between sections. Denser subjects (SaaS, dashboards, admin) → 64-80px. Editorial / luxury → 120-160px.\n\
- motion: ONE signature animation to activate on the hero. MUST be exactly one from these lists, matched to the subject + theme:\n\
    CSS/JS motions:\n\
    · hero-float: soft/luxury/warm/editorial — hero image or headline gently floats.\n\
    · text-stagger: editorial/luxury — hero headline reveals word-by-word (requires words wrapped in <span>).\n\
    · gradient-shift: playful/tech — hero background gradient slowly shifts hue.\n\
    · line-underline: minimal/editorial — an accent underline draws under the primary CTA.\n\
    · marquee-trust: SaaS/consumer — logo or testimonial row scrolls horizontally forever.\n\
    · parallax-hero: editorial/luxury/travel — hero image translates at 0.4× scroll speed.\n\
    · noise-grain: editorial/luxury/premium — subtle SVG grain on the hero.\n\
    · magnetic-cta: playful/tech — primary CTA follows the cursor slightly.\n\
    · orbit-badge: playful/tech — a small decorative badge orbits the hero focal (requires an element with class \"orbit\").\n\
    3D scenes (pick these when the subject warrants a WOW moment — SaaS, tech, luxury, premium, cosmic, developer, spatial):\n\
    · 3d-orb-glow: wireframe icosahedron behind the hero, gently rotates + pulses. Great default for tech / SaaS / spatial subjects.\n\
    · 3d-particle-drift: 1200 accent-colored particles drifting slowly — atmosphere. Editorial / creative / abstract.\n\
    · 3d-wireframe: rotating wireframe torus knot — technical / developer / infrastructure.\n\
    · 3d-gradient-plane: animated shader gradient wash — playful / creative / consumer.\n\
    · 3d-star-field: starfield rotating slowly — cosmic / space / dark-mode / premium.\n\
    · 3d-flowing-lines: eight thin curved ribbons flowing horizontally — luxury / motion / dance / music.\n\
    · none: for extreme minimal/brutalist themes where any motion is a distraction.\n\n\
Use 3D at MOST once per page and only when it adds genuine signature — don't force it on a matrimony or bakery site. All four tweak keys required. No prose. No markdown fences. Just the JSON object.";

    let user = format!(
        "Subject: {idea}\nTone package: {theme_id}{refs_prompt_block}\n\nFill these placeholder keys with concrete values suitable for \"{idea}\":\n\n{}\n\nReturn a JSON object with `placeholders` (every key above must appear once) and `tweaks` (radius, shadow, section_gap).",
        all_placeholders.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n"),
    );

    // Kit-picker path = Landing mode by definition. Assembly LLM outputs
    // JSON, not HTML — DON'T stream chunks to the preview iframe.
    let comp = provider.complete_streaming_cached(
        system,
        ai::prompts::system_context(crate::session::Mode::Landing),
        &user, 3500,
        Box::new(|_| {}),
        stop.clone(),
    ).await?;
    let comp = ai::Completion { text: ai::clean(comp.text), usage: comp.usage };
    let usage = usage_or_estimate(comp.usage, system, &user, &comp.text);

    // The LLM output is now {placeholders: {…}, tweaks: {radius, shadow, section_gap}}.
    // Parse both — falling back gracefully if the model returned only the flat
    // placeholders dict (older shape).
    let (fills, tweaks) = parse_fills_and_tweaks(&comp.text, &all_placeholders);
    let scraped_font = select_scraped_font(&scraped_refs);
    if !tweaks.is_empty() || scraped_font.is_some() {
        let _ = tx.send(AppEvent::StatusUpdate(
            format!("Weaving {} style tweaks{}…",
                tweaks.len(),
                if scraped_font.is_some() { " + scraped font" } else { "" })
        ));
    }

    // Fold the LLM tweaks + scraped font into a fresh palette (owned) so the
    // rest of the pipeline sees a single Palette carrying every override.
    let final_palette = apply_style_tweaks(palette, &tweaks);

    // Signature motion (LLM's pick) rides on the <body> as a data attribute
    // that the motion CSS/JS keys on. Whitelist against the known set.
    let motion_pick = tweaks.get("motion").cloned().unwrap_or_default();
    let allowed = ["hero-float","text-stagger","gradient-shift","line-underline",
                   "marquee-trust","parallax-hero","noise-grain","magnetic-cta",
                   "orbit-badge",
                   "3d-orb-glow","3d-particle-drift","3d-wireframe",
                   "3d-gradient-plane","3d-star-field","3d-flowing-lines",
                   "none"];
    let signature_motion = if allowed.contains(&motion_pick.as_str()) && motion_pick != "none" {
        Some(motion_pick.as_str())
    } else { None };

    // ── Progressive assembly reveal ──
    // Instead of showing the completed HTML in one flash, add sections one at
    // a time to the preview iframe. Each step is a full HTML snapshot with a
    // growing <body>. Small delays make the build feel intentional.
    let (shell_open, shell_close) = assemble_shell_ext(theme, &final_palette, &selected, idea, scraped_font.as_deref());
    // Inject the signature-motion attribute onto <body>.
    let shell_open = if let Some(m) = signature_motion {
        shell_open.replace("<body>\n", &format!("<body data-signature-motion=\"{}\">\n", m))
    } else { shell_open };
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

    // Progressive-assembly reveal is done. Now run the orchestrator's
    // auto-quality pass BEFORE the final DesignUpdate so any high-severity
    // fixes are baked in silently; remaining critique items become polish
    // chips the user can click.
    let assembled = format!("{shell_open}{body_acc}{shell_close}");
    let octx = crate::orchestrator::OrchestratorCtx {
        brief:    idea,
        mode:     crate::session::Mode::Landing,
        provider,
        tx,
        stop:     stop.clone(),
    };
    let (final_html, remaining) =
        crate::orchestrator::auto_quality_pass(&octx, assembled, 1).await;

    let _ = tx.send(AppEvent::DesignUpdate(final_html.clone()));
    let _ = tx.send(AppEvent::AssistantMessage(format!(
        "Assembled from {} sections. Click any section on the canvas — a ↻ swap panel opens on the left for zero-token variant swaps.",
        selected.len()
    )));
    if !remaining.is_empty() {
        let json = serde_json::to_string(&remaining).unwrap_or_else(|_| "[]".into());
        let _ = tx.send(AppEvent::CritiqueFixes(json));
    }

    Ok((
        State::Refining {
            current: final_html,
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
    let (fills, _) = parse_fills_and_tweaks(raw, keys);
    fills
}

/// Parse the placeholder-fill LLM's JSON. Supports two shapes:
///   NEW: {"placeholders": {…}, "tweaks": {radius, shadow, section_gap}}
///   OLD: {"KEY": "value", …}   (flat placeholders dict, no tweaks)
/// Fallbacks fill any missing placeholder keys. Tweaks map may be empty.
fn parse_fills_and_tweaks(raw: &str, keys: &[String]) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut fills:  HashMap<String, String> = HashMap::new();
    let mut tweaks: HashMap<String, String> = HashMap::new();

    let stripped = raw.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    let start = stripped.find('{');
    let end   = stripped.rfind('}');
    if let (Some(a), Some(b)) = (start, end) {
        if b > a {
            let json_str = &stripped[a..=b];
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                // Prefer the new two-key shape when present.
                let ph_obj    = v.get("placeholders").and_then(|x| x.as_object());
                let tweak_obj = v.get("tweaks").and_then(|x| x.as_object());
                let flat_obj  = if ph_obj.is_none() && tweak_obj.is_none() {
                    v.as_object()
                } else { None };

                let source = ph_obj.or(flat_obj);
                if let Some(obj) = source {
                    for (k, val) in obj {
                        let s = val.as_str().map(|s| s.to_string())
                            .unwrap_or_else(|| val.to_string().trim_matches('"').to_string());
                        fills.insert(k.clone(), s);
                    }
                }
                if let Some(obj) = tweak_obj {
                    for (k, val) in obj {
                        let s = val.as_str().map(|s| s.to_string())
                            .unwrap_or_else(|| val.to_string().trim_matches('"').to_string());
                        if !s.is_empty() { tweaks.insert(k.clone(), s); }
                    }
                }
            }
        }
    }
    for k in keys {
        if !fills.contains_key(k) {
            fills.insert(k.clone(), fallback_for(k));
        }
    }
    (fills, tweaks)
}

/// Fold LLM style tweaks into a copy of the base palette. Radius / shadow /
/// section_gap get appended (or replaced) as :root custom properties. Also
/// wraps the shadow value in `var()` fallback so downstream CSS that reads
/// --shadow-elev works verbatim.
fn apply_style_tweaks(base: &crate::variants::Palette, tweaks: &HashMap<String, String>) -> crate::variants::Palette {
    if tweaks.is_empty() { return base.clone(); }
    let mut body = base.body.clone();
    let mut ensure = |name: &str, value: &str| {
        // If the base body already defines this var, overwrite; else append.
        if body.contains(&format!("--{}:", name)) || body.contains(&format!("--{}: ", name)) {
            body = replace_css_var(&body, &format!("--{}", name), value);
        } else {
            if !body.ends_with('\n') { body.push('\n'); }
            body.push_str(&format!("--{}: {};\n", name, value));
        }
    };
    if let Some(v) = tweaks.get("radius")      { ensure("radius",      v); }
    if let Some(v) = tweaks.get("shadow")      { ensure("shadow-elev", v); }
    if let Some(v) = tweaks.get("section_gap") { ensure("section-gap", v); }
    crate::variants::Palette {
        id:   format!("{}+tweaks", base.id),
        tags: base.tags.clone(),
        body,
    }
}

/// Pick the first scraped font that looks like a plausible Google Font
/// family name (not a generic keyword or system stack). Case-insensitive
/// check against the well-known aliases.
fn select_scraped_font(refs: &[scraper::DesignRef]) -> Option<String> {
    let bad = ["system-ui", "-apple-system", "blinkmacsystemfont", "segoe ui", "helvetica",
               "arial", "sans-serif", "serif", "monospace", "inherit", "ui-monospace", "ui-sans-serif"];
    for r in refs {
        for f in &r.fonts {
            let name = f.trim().trim_matches('"').trim_matches('\'').trim().to_string();
            if name.is_empty() { continue; }
            let low = name.to_ascii_lowercase();
            if bad.iter().any(|b| low == *b || low.starts_with(&format!("{}, ", b))) { continue; }
            // Cheap plausibility gate — Google Fonts names are letter-and-space,
            // 3 to 30 chars, start with a letter.
            let ok = name.chars().all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-')
                && name.len() >= 3 && name.len() <= 30
                && name.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false);
            if ok { return Some(name); }
        }
    }
    None
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
// ── Palette blending (kit-picker uniqueness) ─────────────────────────────
//
// Scraped color hex codes get sorted by luminance + saturation and mapped
// onto the base palette's semantic slots (--paper / --ink / --accent). The
// layout structure stays hand-authored; only the color story changes so
// no two kit-picker landing pages look the same.

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}
fn luminance(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}
fn saturation(r: u8, g: u8, b: u8) -> f32 {
    let max = r.max(g).max(b) as f32 / 255.0;
    let min = r.min(g).min(b) as f32 / 255.0;
    if max <= 0.0 { 0.0 } else { (max - min) / max }
}

fn replace_css_var(body: &str, var_name: &str, value: &str) -> String {
    // Replace a `--var: value;` line inside a CSS-property block. Preserves
    // any leading whitespace so the block's formatting stays intact.
    let escaped = regex::escape(var_name);
    let re = match regex::Regex::new(&format!(r"(?m)^([\t ]*){}\s*:\s*[^;]+;", escaped)) {
        Ok(re) => re, Err(_) => return body.to_string(),
    };
    re.replace(body, format!("$1{}: {};", var_name, value).as_str()).to_string()
}

// WCAG relative-luminance contrast ratio (needs sRGB-linear luminance, but a
// simple gamma-approx works well enough for the go/no-go check we care about).
fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    let f = |c: u8| {
        let x = c as f32 / 255.0;
        if x <= 0.03928 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}
fn contrast_ratio(a: (u8,u8,u8), b: (u8,u8,u8)) -> f32 {
    let la = relative_luminance(a.0, a.1, a.2);
    let lb = relative_luminance(b.0, b.1, b.2);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Read the current value of `var_name` from a `:root` body (best-effort:
/// returns the RGB triple if the value is a #hex we can parse).
fn parse_var_hex(body: &str, var_name: &str) -> Option<(u8, u8, u8)> {
    let escaped = regex::escape(var_name);
    let re = regex::Regex::new(&format!(r"(?m)^[\t ]*{}\s*:\s*(#[0-9a-fA-F]{{6}})\s*;", escaped)).ok()?;
    let cap = re.captures(body)?;
    parse_hex(cap.get(1)?.as_str())
}

fn blend_palette(base: &Variant_Palette, scraped_colors: &[String]) -> Variant_Palette {
    let mut valid: Vec<(String, (u8, u8, u8), f32, f32)> = scraped_colors.iter()
        .filter_map(|h| parse_hex(h).map(|rgb| {
            let l = luminance(rgb.0, rgb.1, rgb.2);
            let s = saturation(rgb.0, rgb.1, rgb.2);
            (h.trim().to_string(), rgb, l, s)
        }))
        .collect();
    if valid.len() < 3 { return base.clone(); }
    valid.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // Detect the base palette's polarity so we don't invert light/dark.
    // Dark palettes have a low-luminance --paper; ink must stay light.
    let base_paper = parse_var_hex(&base.body, "--paper");
    let base_ink   = parse_var_hex(&base.body, "--ink");
    let is_dark_palette = base_paper
        .map(|(r,g,b)| relative_luminance(r,g,b) < 0.15)
        .unwrap_or(false);

    let (paper_target_lum_gt, ink_target_lum_lt) = if is_dark_palette {
        // paper must stay dark (< 0.15), ink must stay light (> 0.55)
        (false, false)
    } else {
        // light palette: paper light (> 0.85), ink dark (< 0.30)
        (true, true)
    };

    let mut body = base.body.clone();

    // --paper override
    let paper_candidate = if paper_target_lum_gt {
        valid.iter().rev().find(|v| v.2 > 0.85).cloned()
    } else {
        valid.iter().find(|v| v.2 < 0.15).cloned()
    };
    let mut new_paper_rgb = base_paper;
    if let Some((hex, rgb, _, _)) = paper_candidate {
        body = replace_css_var(&body, "--paper", &hex);
        new_paper_rgb = Some(rgb);
    }

    // --ink override — MUST contrast with the (possibly new) --paper.
    let ink_candidate = if ink_target_lum_lt {
        valid.iter().find(|v| v.2 < 0.30).cloned()
    } else {
        valid.iter().rev().find(|v| v.2 > 0.55).cloned()
    };
    if let Some((hex, rgb, _, _)) = ink_candidate {
        // Contrast gate: only override if the pair still passes AA (4.5:1).
        let paper_ref = new_paper_rgb.or(base_paper).unwrap_or((255,255,255));
        if contrast_ratio(rgb, paper_ref) >= 4.5 {
            body = replace_css_var(&body, "--ink", &hex);
        }
    }

    // --accent: most saturated mid-luminance color; must contrast against paper
    // enough that CTAs are readable (3:1 minimum for large text).
    let paper_for_accent = new_paper_rgb.or(base_paper).unwrap_or((255,255,255));
    let accent = valid.iter()
        .filter(|v| v.2 > 0.15 && v.2 < 0.85 && v.3 > 0.30)
        .filter(|v| contrast_ratio(v.1, paper_for_accent) >= 3.0)
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        .cloned();
    if let Some((hex, (r,g,b), _, _)) = accent {
        body = replace_css_var(&body, "--accent", &hex);
        let darker = format!("#{:02x}{:02x}{:02x}",
            (r as f32 * 0.85) as u8, (g as f32 * 0.85) as u8, (b as f32 * 0.85) as u8);
        body = replace_css_var(&body, "--accent-2", &darker);
        let soft = format!("rgba({},{},{},0.08)", r, g, b);
        body = replace_css_var(&body, "--accent-soft", &soft);
    }

    // Final safety net: if the resulting --ink/--paper pair somehow ended up
    // below AA (e.g. base_ink was borderline and now paper shifted), fall
    // back entirely — an inconsistent-but-readable base beats a pretty-but-
    // illegible blend every time.
    let final_paper = parse_var_hex(&body, "--paper").unwrap_or((255,255,255));
    let final_ink   = parse_var_hex(&body, "--ink").or(base_ink).unwrap_or((0,0,0));
    if contrast_ratio(final_ink, final_paper) < 4.5 {
        return base.clone();
    }

    Variant_Palette {
        id:   format!("{}+blend", base.id),
        tags: base.tags.clone(),
        body,
    }
}

// Alias so the blend fn doesn't need to import from variants:: at each site.
use crate::variants::Palette as Variant_Palette;

fn assemble_shell(
    theme: &Theme, palette: &Palette, sections: &[&Variant], idea: &str,
) -> (String, String) {
    assemble_shell_ext(theme, palette, sections, idea, None)
}

/// Extended shell builder that honors a scraped font override for the
/// display family — the theme's original font stays as fallback so
/// widow characters still render if Google Fonts drops the scraped one.
fn assemble_shell_ext(
    theme: &Theme, palette: &Palette, sections: &[&Variant], idea: &str,
    display_override: Option<&str>,
) -> (String, String) {
    let fonts_meta = theme.meta.get("fonts").cloned().unwrap_or_default();
    let theme_display = extract_font(&fonts_meta, "display").unwrap_or_else(|| "Inter".into());
    let display = display_override
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| theme_display.clone());
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
    // Scraped display font (if any) wins; theme's original stays as fallback
    // so text still renders if the scraped font fails to load.
    if display != theme_display {
        head.push_str(&format!("--font-display: '{}', '{}', Georgia, serif;\n", display, theme_display));
    } else {
        head.push_str(&format!("--font-display: '{}', Georgia, serif;\n", display));
    }
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
    head.push_str(MOTION_CSS);
    head.push_str("</style>\n</head>\n<body>\n");
    head.push_str(MOTION_JS);
    head.push_str(THREEJS_LIBRARY);

    let tail = String::from("</body>\n</html>\n");
    (head, tail)
}

/// Base motion CSS — scroll-triggered reveals (initial state + `.motion-in`),
/// plus a small named library of signature moves that get activated by setting
/// `data-signature-motion="…"` on the body (LLM's choice, one per page).
const MOTION_CSS: &str = r#"
/* ── Scroll-reveal base ── */
section, nav, footer, header, aside, main, article {
  opacity: 0;
  transform: translateY(28px);
  transition:
    opacity 0.7s cubic-bezier(0.16,1,0.3,1),
    transform 0.7s cubic-bezier(0.16,1,0.3,1);
  will-change: opacity, transform;
}
section.motion-in, nav.motion-in, footer.motion-in,
header.motion-in, aside.motion-in, main.motion-in, article.motion-in {
  opacity: 1; transform: translateY(0);
}
/* Nested reveals — direct children of the first hero-like section get a
   subtle stagger without the LLM having to think about it. */
section:first-of-type.motion-in > * {
  animation: staggerIn 0.7s cubic-bezier(0.16,1,0.3,1) both;
}
section:first-of-type.motion-in > *:nth-child(2){ animation-delay: 90ms; }
section:first-of-type.motion-in > *:nth-child(3){ animation-delay: 180ms; }
section:first-of-type.motion-in > *:nth-child(4){ animation-delay: 270ms; }
section:first-of-type.motion-in > *:nth-child(5){ animation-delay: 360ms; }
@keyframes staggerIn {
  from { opacity: 0; transform: translateY(16px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* ── Signature motions (one active per page via body[data-signature-motion]) ── */

/* hero-float: gentle 6s Y-float on the first h1 or hero image */
body[data-signature-motion="hero-float"] section:first-of-type :is(h1, img, .hero-visual, .hero-img) {
  animation: heroFloat 6s ease-in-out infinite;
}
@keyframes heroFloat {
  0%, 100% { transform: translateY(0); }
  50%      { transform: translateY(-10px); }
}

/* text-stagger: word-by-word reveal on the hero h1 (needs LLM to wrap words in spans) */
body[data-signature-motion="text-stagger"] section:first-of-type h1 span {
  display: inline-block;
  opacity: 0; transform: translateY(20px);
  animation: wordIn 0.9s cubic-bezier(0.16,1,0.3,1) both;
}
@keyframes wordIn { to { opacity: 1; transform: translateY(0); } }

/* gradient-shift: slow hue rotation on the hero background */
body[data-signature-motion="gradient-shift"] section:first-of-type {
  background-size: 200% 200%;
  animation: gradientShift 18s ease-in-out infinite;
}
@keyframes gradientShift {
  0%, 100% { background-position: 0% 50%; }
  50%      { background-position: 100% 50%; }
}

/* line-underline: accent underline draws in on the primary CTA */
body[data-signature-motion="line-underline"] section:first-of-type a[href]:first-of-type,
body[data-signature-motion="line-underline"] section:first-of-type button:first-of-type {
  position: relative;
}
body[data-signature-motion="line-underline"] section:first-of-type a[href]:first-of-type::after,
body[data-signature-motion="line-underline"] section:first-of-type button:first-of-type::after {
  content: '';
  position: absolute; left: 6%; right: 6%; bottom: -6px; height: 2px;
  background: var(--accent);
  transform-origin: left center;
  animation: lineDraw 1.2s cubic-bezier(0.16,1,0.3,1) 0.4s both;
}
@keyframes lineDraw {
  from { transform: scaleX(0); }
  to   { transform: scaleX(1); }
}

/* marquee-trust: horizontal loop for logo/testimonial rows tagged with .marquee */
.marquee {
  overflow: hidden; -webkit-mask-image: linear-gradient(90deg, transparent, #000 8%, #000 92%, transparent);
          mask-image: linear-gradient(90deg, transparent, #000 8%, #000 92%, transparent);
}
.marquee-inner { display: inline-flex; gap: 48px; animation: marqueeSlide 26s linear infinite; }
@keyframes marqueeSlide { from { transform: translateX(0); } to { transform: translateX(-50%); } }

/* parallax-hero: hero image translates half the scroll distance (JS driven) */
body[data-signature-motion="parallax-hero"] section:first-of-type :is(img, .hero-img) {
  will-change: transform;
}

/* noise-grain: subtle SVG noise overlay on the whole hero */
body[data-signature-motion="noise-grain"] section:first-of-type {
  position: relative;
}
body[data-signature-motion="noise-grain"] section:first-of-type::before {
  content: ''; position: absolute; inset: 0; pointer-events: none; z-index: 1; opacity: 0.06;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='240' height='240'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2'/><feColorMatrix values='0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0'/></filter><rect width='240' height='240' filter='url(%23n)'/></svg>");
  mix-blend-mode: overlay;
}

/* magnetic-cta: primary CTA follows cursor slightly (JS driven) */
body[data-signature-motion="magnetic-cta"] section:first-of-type :is(a[href], button):first-of-type {
  transition: transform 0.2s cubic-bezier(0.16,1,0.3,1);
  will-change: transform;
}

/* orbit-badge: floating decorative badge orbits the hero focal (LLM adds .orbit el) */
body[data-signature-motion="orbit-badge"] .orbit {
  animation: orbit 24s linear infinite;
  transform-origin: -80px center;
}
@keyframes orbit { to { transform: rotate(360deg); } }

/* Respect user preference */
@media (prefers-reduced-motion: reduce) {
  section, nav, footer, header, aside, main, article {
    opacity: 1 !important; transform: none !important; transition: none !important;
  }
  body[data-signature-motion] * { animation: none !important; }
}
"#;

/// three.js scene library — six pre-authored signature scenes, injected into
/// the hero when the LLM picks a `3d-*` signature-motion. The container is
/// absolute-positioned behind the hero content with pointer-events:none so
/// interactions still land on the actual UI. Colors read from the palette
/// custom properties so scenes always match the blended palette.
const THREEJS_LIBRARY: &str = r#"
<script type="importmap">
{"imports":{"three":"https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js"}}
</script>
<script type="module">
(function(){
  var sig = document.body.getAttribute('data-signature-motion') || '';
  if (!sig.startsWith('3d-')) return;
  if (matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  // Read palette hex from CSS custom properties. Falls back to a warm brick.
  function cssHex(name, fallback) {
    var v = getComputedStyle(document.body).getPropertyValue(name).trim();
    return (v.length >= 4 && v[0] === '#') ? v : fallback;
  }
  var ACCENT = cssHex('--accent',  '#d76146');
  var INK    = cssHex('--ink',     '#1c1a17');
  var PAPER  = cssHex('--paper',   '#f5efe4');

  // Container: sits behind the hero, non-interactive, follows hero size.
  var hero = document.querySelector('section:first-of-type, header:first-of-type');
  if (!hero) return;
  var host = document.createElement('div');
  host.id = '__sig_scene';
  host.style.cssText = 'position:absolute;inset:0;z-index:0;pointer-events:none;overflow:hidden;';
  if (getComputedStyle(hero).position === 'static') hero.style.position = 'relative';
  hero.insertBefore(host, hero.firstChild);
  // Ensure hero content sits above the scene.
  var kids = hero.children;
  for (var i = 0; i < kids.length; i++) {
    if (kids[i] === host) continue;
    if (getComputedStyle(kids[i]).position === 'static') kids[i].style.position = 'relative';
    kids[i].style.zIndex = '1';
  }

  import('three').then(function(THREE) {
    var renderer = new THREE.WebGLRenderer({ alpha:true, antialias:true, powerPreference:'high-performance' });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);
    var scene = new THREE.Scene();
    var camera = new THREE.PerspectiveCamera(50, 1, 0.1, 100);
    camera.position.z = 4;
    host.appendChild(renderer.domElement);

    function resize() {
      var w = host.clientWidth, h = host.clientHeight;
      if (w < 10 || h < 10) return;
      renderer.setSize(w, h, false);
      camera.aspect = w / h; camera.updateProjectionMatrix();
    }
    window.addEventListener('resize', resize);
    resize();

    // Helper: hex → THREE.Color
    function C(h) { return new THREE.Color(h); }

    var mesh, points, uniforms, paused = false, rafId = null;
    var scenes = {
      'orb-glow': function(){
        var geo = new THREE.IcosahedronGeometry(1.4, 1);
        var mat = new THREE.MeshBasicMaterial({ color: C(ACCENT), wireframe: true, transparent: true, opacity: 0.55 });
        mesh = new THREE.Mesh(geo, mat);
        scene.add(mesh);
        // Soft additive halo — a second, larger, fainter mesh.
        var glow = new THREE.Mesh(
          new THREE.IcosahedronGeometry(1.9, 1),
          new THREE.MeshBasicMaterial({ color: C(ACCENT), wireframe: true, transparent: true, opacity: 0.15 })
        );
        scene.add(glow);
        return function tick(t){
          mesh.rotation.x = t * 0.00015;
          mesh.rotation.y = t * 0.00022;
          mesh.scale.setScalar(1 + Math.sin(t * 0.0008) * 0.06);
          glow.rotation.x = -t * 0.0001;
          glow.rotation.y = -t * 0.00013;
          renderer.render(scene, camera);
        };
      },
      'particle-drift': function(){
        var N = 1200;
        var positions = new Float32Array(N * 3);
        var speeds = new Float32Array(N);
        for (var i = 0; i < N; i++) {
          positions[i*3+0] = (Math.random() - 0.5) * 10;
          positions[i*3+1] = (Math.random() - 0.5) * 6;
          positions[i*3+2] = (Math.random() - 0.5) * 4;
          speeds[i] = 0.002 + Math.random() * 0.003;
        }
        var geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        var mat = new THREE.PointsMaterial({ color: C(ACCENT), size: 0.03, transparent: true, opacity: 0.45, sizeAttenuation: true });
        points = new THREE.Points(geo, mat);
        scene.add(points);
        return function tick(t){
          var pos = geo.attributes.position.array;
          for (var i = 0; i < N; i++) {
            pos[i*3+0] += speeds[i];
            if (pos[i*3+0] > 5) pos[i*3+0] = -5;
            pos[i*3+1] += Math.sin((t + i * 100) * 0.0005) * 0.001;
          }
          geo.attributes.position.needsUpdate = true;
          renderer.render(scene, camera);
        };
      },
      'wireframe': function(){
        var geo = new THREE.TorusKnotGeometry(1.1, 0.32, 200, 20);
        var mat = new THREE.MeshBasicMaterial({ color: C(ACCENT), wireframe: true, transparent: true, opacity: 0.6 });
        mesh = new THREE.Mesh(geo, mat);
        scene.add(mesh);
        return function tick(t){
          mesh.rotation.x = t * 0.00018;
          mesh.rotation.y = t * 0.00024;
          renderer.render(scene, camera);
        };
      },
      'gradient-plane': function(){
        uniforms = {
          time: { value: 0 },
          c1:   { value: C(ACCENT) },
          c2:   { value: C(PAPER) },
          c3:   { value: C(INK) }
        };
        var mat = new THREE.ShaderMaterial({
          uniforms: uniforms,
          vertexShader: 'varying vec2 vUv; void main(){ vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }',
          fragmentShader:
            'uniform float time; uniform vec3 c1; uniform vec3 c2; uniform vec3 c3; varying vec2 vUv; ' +
            'void main(){ ' +
              'float n = sin(vUv.x * 3.14 + time * 0.4) * 0.5 + 0.5; ' +
              'float m = cos(vUv.y * 2.71 - time * 0.3) * 0.5 + 0.5; ' +
              'vec3 col = mix(c2, c1, n); col = mix(col, c3, m * 0.15); ' +
              'gl_FragColor = vec4(col, 0.35); }',
          transparent: true,
        });
        mesh = new THREE.Mesh(new THREE.PlaneGeometry(20, 12), mat);
        scene.add(mesh);
        camera.position.z = 3;
        return function tick(t){
          uniforms.time.value = t * 0.001;
          renderer.render(scene, camera);
        };
      },
      'star-field': function(){
        var N = 900;
        var positions = new Float32Array(N * 3);
        var sizes = new Float32Array(N);
        for (var i = 0; i < N; i++) {
          var r = 6 + Math.random() * 4;
          var theta = Math.random() * Math.PI * 2;
          var phi = Math.acos(2 * Math.random() - 1);
          positions[i*3+0] = r * Math.sin(phi) * Math.cos(theta);
          positions[i*3+1] = r * Math.sin(phi) * Math.sin(theta);
          positions[i*3+2] = r * Math.cos(phi) - 6;
          sizes[i] = 0.005 + Math.random() * 0.025;
        }
        var geo = new THREE.BufferGeometry();
        geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geo.setAttribute('size',     new THREE.BufferAttribute(sizes, 1));
        var mat = new THREE.PointsMaterial({ color: 0xffffff, size: 0.02, transparent: true, opacity: 0.7 });
        points = new THREE.Points(geo, mat);
        scene.add(points);
        return function tick(t){
          points.rotation.y = t * 0.00005;
          points.rotation.x = t * 0.00003;
          // Twinkle via material opacity oscillation.
          mat.opacity = 0.55 + Math.sin(t * 0.001) * 0.15;
          renderer.render(scene, camera);
        };
      },
      'flowing-lines': function(){
        // Ribbon-like lines using thin extruded curves.
        var group = new THREE.Group();
        var LINES = 8;
        for (var l = 0; l < LINES; l++) {
          var pts = [];
          for (var s = 0; s < 60; s++) {
            var x = (s / 60 - 0.5) * 8;
            var y = Math.sin(s * 0.22 + l * 0.9) * (0.8 - l * 0.06);
            var z = -l * 0.3;
            pts.push(new THREE.Vector3(x, y, z));
          }
          var curve = new THREE.CatmullRomCurve3(pts);
          var geo = new THREE.TubeGeometry(curve, 60, 0.02 + l * 0.005, 8, false);
          var mat = new THREE.MeshBasicMaterial({ color: C(ACCENT), transparent: true, opacity: 0.25 + l * 0.03 });
          group.add(new THREE.Mesh(geo, mat));
        }
        scene.add(group);
        mesh = group;
        return function tick(t){
          group.rotation.z = Math.sin(t * 0.0003) * 0.15;
          group.position.y = Math.sin(t * 0.0005) * 0.25;
          renderer.render(scene, camera);
        };
      }
    };

    var sceneKey = sig.replace(/^3d-/, '');
    var loop = (scenes[sceneKey] || scenes['orb-glow'])();

    function frame(t){
      if (paused) { rafId = requestAnimationFrame(frame); return; }
      loop(t);
      rafId = requestAnimationFrame(frame);
    }
    document.addEventListener('visibilitychange', function(){ paused = document.hidden; });
    rafId = requestAnimationFrame(frame);
  }).catch(function(err){
    // three.js CDN blocked or offline — degrade silently, no scene shown.
    host.remove();
  });
})();
</script>
"#;

/// Motion JS — small IIFE that installs the IntersectionObserver reveal and
/// activates the parallax + magnetic-CTA signatures when they're picked.
const MOTION_JS: &str = r#"
<script>
(function(){
  var reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  if (reduced) {
    document.querySelectorAll('section, nav, footer, header, aside, main, article')
      .forEach(function(el){ el.classList.add('motion-in'); });
    return;
  }
  // Scroll-triggered reveal.
  var io = new IntersectionObserver(function(entries){
    entries.forEach(function(e){
      if (e.isIntersecting) {
        e.target.classList.add('motion-in');
        io.unobserve(e.target);
      }
    });
  }, { rootMargin: '0px 0px -8% 0px', threshold: 0.08 });
  document.querySelectorAll('section, nav, footer, header, aside, main, article')
    .forEach(function(el){ io.observe(el); });

  var sig = document.body.getAttribute('data-signature-motion') || '';

  // parallax-hero: translate hero image slower than scroll.
  if (sig === 'parallax-hero') {
    var hero = document.querySelector('section:first-of-type img, section:first-of-type .hero-img');
    if (hero) {
      var raf = null;
      var onScroll = function(){
        if (raf) return;
        raf = requestAnimationFrame(function(){
          var y = window.scrollY * 0.4;
          hero.style.transform = 'translate3d(0, ' + y + 'px, 0)';
          raf = null;
        });
      };
      window.addEventListener('scroll', onScroll, { passive: true });
    }
  }

  // magnetic-cta: primary CTA translates slightly toward the cursor.
  if (sig === 'magnetic-cta') {
    var cta = document.querySelector('section:first-of-type a[href], section:first-of-type button');
    if (cta) {
      cta.addEventListener('mousemove', function(e){
        var r = cta.getBoundingClientRect();
        var dx = (e.clientX - (r.left + r.width/2)) * 0.15;
        var dy = (e.clientY - (r.top + r.height/2)) * 0.25;
        cta.style.transform = 'translate3d(' + dx + 'px, ' + dy + 'px, 0)';
      });
      cta.addEventListener('mouseleave', function(){
        cta.style.transform = '';
      });
    }
  }
})();
</script>
"#;

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
/// Skeleton-first APP flow (v0.6). Generates wireframes for every declared
/// page — INCLUDING home — using the LLM to invent a consistent app shell
/// (sidebar/topbar) shared across all pages. Returns the home skeleton HTML
/// (which becomes the initial `DesignUpdate`) along with usage.
async fn handle_app_skeleton_phase(
    project_slug: &str,
    idea:         &str,
    other_pages:  &[String],
    provider:     &Provider,
    tx:           &Sender<AppEvent>,
    stop:         Arc<AtomicBool>,
) -> Result<(String, ai::Usage)> {
    // Compose the page list: home + others. Slugify names, dedupe.
    let mut list: Vec<(String, String)> = Vec::new();
    list.push(("home".into(), "Home".into()));
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::from(["home".into()]);
    for raw in other_pages {
        let name = raw.trim();
        if name.is_empty() { continue; }
        let slug = projects::slugify(name);
        if slug.is_empty() || !seen.insert(slug.clone()) { continue; }
        list.push((slug, name.to_string()));
    }
    if list.len() < 2 {
        // No sibling pages inferred — fall back to a home-only wireframe.
        // Still useful so the user can iterate the shell before building.
    }

    let _ = tx.send(AppEvent::StatusUpdate(
        format!("Wireframing {} pages…", list.len())
    ));

    // Seed the manifest so tabs appear immediately.
    let mut manifest = projects::PagesManifest {
        pages: list.iter().map(|(slug, name)| projects::PageInfo {
            slug: slug.clone(), name: name.clone(),
            built: false, has_skeleton: false,
        }).collect(),
        active: "home".into(),
    };
    let _ = projects::write_pages_manifest(project_slug, &manifest);
    push_pages(tx, project_slug);

    // Batch LLM call — one round-trip generates every wireframe.
    let system = ai::prompts::SKELETON_BATCH_SYSTEM;
    let user   = ai::prompts::skeleton_batch_user_fresh(idea, &list);
    let comp = provider.complete_streaming_cached(
        system,
        ai::prompts::system_context(crate::session::Mode::App),
        &user, 14_000,
        Box::new(|_| {}),
        stop,
    ).await?;
    let text = ai::clean(comp.text);
    let usage = usage_or_estimate(comp.usage, system, &user, &text);

    // Parse the {pages: [{slug, html}, …]} envelope.
    let stripped = text.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    let value: serde_json::Value = serde_json::from_str(stripped)
        .map_err(|e| anyhow::anyhow!("skeleton batch JSON parse failed: {e}"))?;
    let arr = value.get("pages").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("skeleton batch missing `pages` array"))?;

    let mut home_html: Option<String> = None;
    let total = arr.len() as u32;
    let mut done = 0u32;
    for entry in arr {
        let slug = entry.get("slug").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        let html = entry.get("html").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if slug.is_empty() || html.is_empty() { continue; }
        let _ = projects::write_skeleton(project_slug, &slug, &html);
        if let Some(p) = manifest.pages.iter_mut().find(|p| p.slug == slug) {
            p.has_skeleton = true;
        }
        if slug == "home" { home_html = Some(html); }
        done += 1;
        let _ = tx.send(AppEvent::SkeletonProgress {
            current: done, total, page: slug,
        });
    }
    let _ = projects::write_pages_manifest(project_slug, &manifest);
    push_pages(tx, project_slug);
    if let Ok(m) = projects::read_pages_manifest(project_slug) {
        let json = serde_json::to_string(&m.pages).unwrap_or_else(|_| "[]".into());
        let _ = tx.send(AppEvent::SkeletonsReady { pages: json });
    }

    let home = home_html.ok_or_else(||
        anyhow::anyhow!("wireframe batch didn't include a home page"))?;
    Ok((home, usage))
}

/// Cheap LLM call: given only the brief (no HTML yet), suggest 4 sibling
/// pages this app would plausibly have. Used by APP-mode start_design when
/// the user skipped the pre-page picker.
async fn suggest_pages_from_brief(provider: &Provider, idea: &str) -> Vec<String> {
    let system = ai::prompts::NEXT_PAGES_SUGGEST_SYSTEM;
    let user   = ai::prompts::next_pages_suggest_user(idea, &[]);
    let comp = match provider.complete(system, &user, 200).await {
        Ok(c) => c, Err(_) => return Vec::new(),
    };
    let cleaned = ai::clean(comp.text);
    let json_str = cleaned.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    #[derive(serde::Deserialize)]
    struct Suggestion { name: String, #[allow(dead_code)] slug: String }
    let parsed: Vec<Suggestion> = serde_json::from_str(json_str).unwrap_or_default();
    parsed.into_iter().map(|s| s.name).take(4).collect()
}

// ── Batch skeleton generation (APP mode) ───────────────────────────────
//
// Fire-and-forget spawner. Fires ONE LLM call that returns a wireframe HTML
// per requested page, saves each to `{slug}--{page}.skeleton.html`, updates
// the manifest's `has_skeleton` flag, and emits `SkeletonsReady` so the UI
// can show dashed-tab indicators. Any errors are surfaced as a chat note but
// don't block anything else.
fn spawn_skeleton_batch(
    project_slug: String,
    idea:         String,
    home_html:    String,
    page_names:   Vec<String>,
    provider:     Provider,
    tx:           Sender<AppEvent>,
    stop:         Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        let _ = generate_skeletons(
            &project_slug, &idea, &home_html, &page_names, &provider, &tx, stop,
        ).await;
    });
}

async fn generate_skeletons(
    project_slug: &str,
    idea:         &str,
    home_html:    &str,
    page_names:   &[String],
    provider:     &Provider,
    tx:           &Sender<AppEvent>,
    stop:         Arc<AtomicBool>,
) -> Result<()> {
    // Resolve each declared name to (slug, name), skipping "home" and
    // anything already in the manifest.
    let mut manifest = projects::read_pages_manifest(project_slug).unwrap_or_default();
    let mut to_wireframe: Vec<(String, String)> = Vec::new();
    for raw in page_names {
        let name = raw.trim();
        if name.is_empty() { continue; }
        let slug = projects::slugify(name);
        if slug == "home" || slug.is_empty() { continue; }
        if manifest.pages.iter().any(|p| p.slug == slug && p.built) { continue; }
        // Add to manifest as skeleton entry if not present.
        if !manifest.pages.iter().any(|p| p.slug == slug) {
            manifest.pages.push(projects::PageInfo {
                slug: slug.clone(), name: name.to_string(),
                built: false, has_skeleton: false,
            });
        }
        to_wireframe.push((slug, name.to_string()));
    }
    if to_wireframe.is_empty() { return Ok(()); }
    // Write the extended manifest so the UI sees the skeleton entries even
    // before the LLM finishes.
    let _ = projects::write_pages_manifest(project_slug, &manifest);
    push_pages(tx, project_slug);
    let _ = tx.send(AppEvent::StatusUpdate(
        format!("Wireframing {} pages…", to_wireframe.len())
    ));

    // One LLM call, JSON output. Non-streaming — the JSON is large and we
    // don't want raw text flashing in the preview iframe.
    let system = ai::prompts::SKELETON_BATCH_SYSTEM;
    let user   = ai::prompts::skeleton_batch_user(idea, home_html, &to_wireframe);

    let comp = provider.complete_streaming_cached(
        system,
        ai::prompts::system_context(crate::session::Mode::App),
        &user, 12_000,
        Box::new(|_| {}),
        stop,
    ).await?;
    let text = ai::clean(comp.text);

    // Parse the {pages: [{slug, html}, …]} envelope. Tolerant to leading
    // whitespace and code fences.
    let stripped = text.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```").trim();
    let value: serde_json::Value = serde_json::from_str(stripped)
        .map_err(|e| anyhow::anyhow!("skeleton batch JSON parse failed: {e}"))?;
    let arr = value.get("pages").and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("skeleton batch missing `pages` array"))?;

    let total = to_wireframe.len() as u32;
    let mut done = 0u32;
    for entry in arr {
        let slug = entry.get("slug").and_then(|v| v.as_str()).unwrap_or("").trim();
        let html = entry.get("html").and_then(|v| v.as_str()).unwrap_or("").trim();
        if slug.is_empty() || html.is_empty() { continue; }
        let _ = projects::write_skeleton(project_slug, slug, html);
        // Reload manifest and mark has_skeleton = true for this slug.
        if let Ok(mut m) = projects::read_pages_manifest(project_slug) {
            if let Some(p) = m.pages.iter_mut().find(|p| p.slug == slug) {
                p.has_skeleton = true;
            }
            let _ = projects::write_pages_manifest(project_slug, &m);
        }
        done += 1;
        let _ = tx.send(AppEvent::SkeletonProgress {
            current: done, total, page: slug.to_string(),
        });
    }

    // Final push so the UI's tab bar reflects the has_skeleton flags.
    push_pages(tx, project_slug);
    // Compact SkeletonsReady payload with the current manifest so the UI can
    // render dashed tabs without a manifest re-read roundtrip.
    if let Ok(m) = projects::read_pages_manifest(project_slug) {
        let json = serde_json::to_string(&m.pages).unwrap_or_else(|_| "[]".into());
        let _ = tx.send(AppEvent::SkeletonsReady { pages: json });
    }
    let _ = tx.send(AppEvent::AssistantMessage(format!(
        "Wireframed {done} pages. Click any tab to preview — hit \"Build this page\" to upgrade a wireframe to full fidelity."
    )));
    Ok(())
}

fn spawn_critique(html: String, mode: crate::session::Mode, provider: Provider, tx: Sender<AppEvent>) {
    tokio::spawn(async move {
        let _ = run_critique(html, mode, provider, tx).await;
    });
}

async fn run_critique(html: String, mode: crate::session::Mode, provider: Provider, tx: Sender<AppEvent>) -> Result<()> {
    // Cap the input HTML so the critique input stays bounded.
    let bounded = bound_html(&html, 30_000);
    let system  = ai::prompts::CRITIQUE_SYSTEM;
    let user    = ai::prompts::critique_user(&bounded);

    // Non-streaming — critique is small (~500 tokens output). Routes through
    // the cached channel so the design-knowledge prefix is reused from the
    // just-completed generation (same 5-minute cache window). Uses the
    // session's mode so the cached prefix matches the design pass.
    let stop = Arc::new(AtomicBool::new(false));
    let comp = provider.complete_streaming_cached(
        system, ai::prompts::system_context(mode), &user, 900,
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
