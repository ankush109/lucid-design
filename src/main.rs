mod ai;
mod config;
mod knowledge;
mod pipeline;
mod projects;
mod scraper;
mod variants;

use anyhow::Result;
use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tokio::sync::mpsc as tokio_mpsc;
use wry::WebViewBuilder;

fn main() -> Result<()> {
    let cfg = match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n[design-gen] Configuration error:\n{}\n", e);
            std::process::exit(1);
        }
    };
    println!("[design-gen] provider={} model={}", cfg.provider, cfg.model);

    // Warm up the variant library so parse errors show at startup, not on
    // first design. Print counts so it's obvious the catalog loaded.
    let lib = variants::library();
    let cats: Vec<String> = lib.by_category.iter()
        .map(|(k, v)| format!("{}:{}", k, v.len()))
        .collect();
    println!(
        "[design-gen] library: {} variants ({}), {} palettes, {} themes",
        lib.variants.len(), cats.join(", "), lib.palettes.len(), lib.themes.len()
    );

    // Install the frontend-design skill for Claude Code CLI so it also has the
    // same design context when invoked directly (not through this app).
    install_frontend_design_skill();

    let provider = ai::build_provider(
        &cfg.provider, &cfg.api_key, &cfg.model, cfg.base_url.as_deref(),
    )?;

    let (ipc_tx, ipc_rx) = tokio_mpsc::unbounded_channel::<pipeline::IpcMessage>();
    let (ui_tx, ui_rx)   = mpsc::channel::<pipeline::AppEvent>();
    let stop_flag        = Arc::new(AtomicBool::new(false));

    let stop_for_thread = stop_flag.clone();
    let provider_id_for_thread = cfg.provider.clone();
    let model_for_thread       = cfg.model.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(pipeline::run(ipc_rx, ui_tx, provider, provider_id_for_thread, model_for_thread, stop_for_thread));
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Design Generator")
        .with_inner_size(LogicalSize::new(1280.0_f64, 820.0_f64))
        .build(&event_loop)?;

    #[cfg(target_os = "macos")]
    install_mac_menu();

    let stop_for_ipc = stop_flag.clone();
    let webview = WebViewBuilder::new(&window)
        .with_devtools(true)
        .with_html(include_str!("assets/ui.html"))
        .with_ipc_handler(move |req: wry::http::Request<String>| {
            let body = req.body();
            if let Ok(msg) = serde_json::from_str::<pipeline::IpcMessage>(body) {
                if msg.kind == "stop_generation" {
                    stop_for_ipc.store(true, Ordering::SeqCst);
                }
                let _ = ipc_tx.send(msg);
            }
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        while let Ok(ev) = ui_rx.try_recv() {
            let json = match &ev {
                pipeline::AppEvent::AssistantMessage(t) => {
                    let cut = t.char_indices().take(60).last().map(|(i, c)| i + c.len_utf8()).unwrap_or(0);
                    eprintln!("[ui] assistant: {}", &t[..cut]);
                    serde_json::json!({"type":"assistant","content":t})
                }
                pipeline::AppEvent::StatusUpdate(t) => {
                    eprintln!("[ui] status: {}", t);
                    serde_json::json!({"type":"status","content":t})
                }
                pipeline::AppEvent::DesignUpdate(h) => {
                    eprintln!("[ui] design: {} bytes", h.len());
                    serde_json::json!({"type":"design","content":h})
                }
                pipeline::AppEvent::ThinkingChunk(c) => {
                    eprintln!("[ui] chunk: {} chars (total acc ~{})", c.len(), c.len());
                    serde_json::json!({"type":"chunk","content":c})
                }
                pipeline::AppEvent::SetGenerating(v) =>
                    serde_json::json!({"type":"generating","value":v}),
                pipeline::AppEvent::ExportDesign(h) => {
                    let path = save_file("design-export", h).unwrap_or_else(|e| e.to_string());
                    serde_json::json!({"type":"assistant","content":format!("Exported: {}", path)})
                }
                pipeline::AppEvent::ExportPrototype(h) => {
                    let path = save_file("design-prototype", h).unwrap_or_else(|e| e.to_string());
                    serde_json::json!({"type":"assistant","content":format!("Prototype saved: {}", path)})
                }
                pipeline::AppEvent::ProjectsList(items) => {
                    let value: serde_json::Value = serde_json::from_str(items)
                        .unwrap_or_else(|_| serde_json::json!([]));
                    serde_json::json!({"type":"projects","items":value})
                }
                pipeline::AppEvent::ProjectOpened { slug, name, html, chat } => {
                    let chat_value: serde_json::Value = serde_json::from_str(chat)
                        .unwrap_or_else(|_| serde_json::json!([]));
                    serde_json::json!({
                        "type":"project_opened",
                        "slug":slug,"name":name,"html":html,
                        "chat":chat_value
                    })
                }
                pipeline::AppEvent::Meta { provider, model } => {
                    serde_json::json!({"type":"meta","provider":provider,"model":model})
                }
                pipeline::AppEvent::AssemblyPreview(html) => {
                    serde_json::json!({"type":"assembly_preview","content":html})
                }
                pipeline::AppEvent::CritiqueFixes(items) => {
                    let value: serde_json::Value = serde_json::from_str(items)
                        .unwrap_or_else(|_| serde_json::json!([]));
                    serde_json::json!({"type":"critique","items":value})
                }
                pipeline::AppEvent::TokenUsage {
                    turn_input, turn_output,
                    session_input, session_output, estimated,
                } => {
                    serde_json::json!({
                        "type":"tokens",
                        "turn_input":turn_input,"turn_output":turn_output,
                        "session_input":session_input,"session_output":session_output,
                        "estimated":estimated
                    })
                }
            };
            let script = format!("window.__onEvent({});", json);
            if let Err(e) = webview.evaluate_script(&script) {
                eprintln!("[ui] evaluate_script error: {:?}", e);
            }
        }

        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            *control_flow = ControlFlow::Exit;
        }
    });
}

#[cfg(target_os = "macos")]
fn install_mac_menu() {
    use cocoa::appkit::{NSApp, NSApplication, NSMenu, NSMenuItem};
    use cocoa::base::nil;
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();

        let main_menu: cocoa::base::id = msg_send![class!(NSMenu), new];

        // ── App menu (macOS convention: first submenu) ──
        let app_item: cocoa::base::id = msg_send![class!(NSMenuItem), new];
        let app_menu: cocoa::base::id = msg_send![class!(NSMenu), new];

        let quit_title = NSString::alloc(nil).init_str("Quit Design Generator");
        let quit_key   = NSString::alloc(nil).init_str("q");
        let quit_item: cocoa::base::id = msg_send![NSMenuItem::alloc(nil),
            initWithTitle: quit_title
            action: sel!(terminate:)
            keyEquivalent: quit_key];
        let _: () = msg_send![app_menu, addItem: quit_item];
        let _: () = msg_send![app_item, setSubmenu: app_menu];
        let _: () = msg_send![main_menu, addItem: app_item];

        // ── Edit menu ──
        let edit_item: cocoa::base::id = msg_send![class!(NSMenuItem), new];
        let edit_title = NSString::alloc(nil).init_str("Edit");
        let edit_menu: cocoa::base::id = msg_send![NSMenu::alloc(nil), initWithTitle: edit_title];
        let _: () = msg_send![edit_item, setTitle: edit_title];

        let add = |title: &str, action: objc::runtime::Sel, key: &str| {
            let t = NSString::alloc(nil).init_str(title);
            let k = NSString::alloc(nil).init_str(key);
            let it: cocoa::base::id = msg_send![NSMenuItem::alloc(nil),
                initWithTitle: t action: action keyEquivalent: k];
            let _: () = msg_send![edit_menu, addItem: it];
        };

        add("Undo",   sel!(undo:),   "z");
        add("Redo",   sel!(redo:),   "Z"); // capital Z ⇒ Shift+Cmd+Z
        let sep: cocoa::base::id = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![edit_menu, addItem: sep];
        add("Cut",        sel!(cut:),       "x");
        add("Copy",       sel!(copy:),      "c");
        add("Paste",      sel!(paste:),     "v");
        add("Select All", sel!(selectAll:), "a");

        let _: () = msg_send![edit_item, setSubmenu: edit_menu];
        let _: () = msg_send![main_menu, addItem: edit_item];

        app.setMainMenu_(main_menu);
    }
}

/// Install the frontend-design skill so Claude Code CLI has design principles
/// available in any session (via `/frontend-design` or the natural skill
/// auto-discovery). Silently no-ops if ~/.claude/skills/ can't be written.
fn install_frontend_design_skill() {
    let home = match std::env::var("HOME") { Ok(h) => h, Err(_) => return };
    let skill_dir = std::path::PathBuf::from(&home)
        .join(".claude").join("skills").join("frontend-design");
    if std::fs::create_dir_all(&skill_dir).is_err() { return; }

    let skill_file = skill_dir.join("SKILL.md");
    let content = format!(
        "---\n\
name: frontend-design\n\
description: Senior UI/UX design principles for building high-quality frontend interfaces — spacing scale, tinted-neutral color systems, ratio-derived typography, layout archetypes (bento / editorial / split / single column / sidebar / z-pattern / masonry), motion, accessibility floor, and the AI-design tells to avoid. Use when producing or reviewing HTML/CSS interfaces.\n\
---\n\n{}",
        ai::prompts::SYSTEM_CONTEXT
    );

    match std::fs::write(&skill_file, &content) {
        Ok(_)  => println!("[design-gen] frontend-design skill installed at {}", skill_file.display()),
        Err(e) => eprintln!("[design-gen] could not install frontend-design skill: {}", e),
    }
}

fn save_file(prefix: &str, html: &str) -> Result<String> {
    let name = format!(
        "{}-{}.html",
        prefix,
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs()
    );
    std::fs::write(&name, html)?;
    Ok(name)
}
