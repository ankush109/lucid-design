// Builds the React frontend before the Rust binary compiles, then embeds the
// single-file `frontend/dist/index.html` via `include_str!` in main.rs.
//
// Skip the frontend build with LUCID_SKIP_FRONTEND_BUILD=1 for fast
// Rust-only iteration when the frontend hasn't changed.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let frontend = Path::new(&manifest_dir).join("frontend");

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/index.html");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/vite.config.js");
    println!("cargo:rerun-if-env-changed=LUCID_SKIP_FRONTEND_BUILD");

    if std::env::var("LUCID_SKIP_FRONTEND_BUILD").is_ok() {
        return;
    }

    let dist = frontend.join("dist").join("index.html");
    let src_dir = frontend.join("src");
    let package_json = frontend.join("package.json");

    // Ship-time contract: the published crate carries a prebuilt
    // frontend/dist/index.html. If the frontend source isn't present (crates.io
    // download without src), we MUST trust the shipped dist and skip npm — a
    // downstream `cargo install` shouldn't require Node.
    if !src_dir.exists() || !package_json.exists() {
        assert!(dist.exists(),
            "frontend/dist/index.html is missing and no frontend source is present. \
             Cannot build without either the prebuilt bundle or the source.");
        return;
    }

    // Local dev: source is present, so build the frontend.
    let node_modules = frontend.join("node_modules");
    if !node_modules.exists() {
        let has_lock = frontend.join("package-lock.json").exists();
        let subcmd = if has_lock { "ci" } else { "install" };
        let status = Command::new("npm")
            .arg(subcmd)
            .current_dir(&frontend)
            .status()
            .expect("failed to run npm — is Node installed?");
        assert!(status.success(), "npm {subcmd} failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(&frontend)
        .status()
        .expect("failed to run npm run build");
    assert!(status.success(), "vite build failed");

    assert!(dist.exists(), "frontend/dist/index.html was not produced");
}
