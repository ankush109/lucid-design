// Per-project Session — mode + brief + tabs + tokens + timestamps.
// Persisted to `{project_dir}/{slug}.session.json`. Every project has its own
// isolated session; switching projects atomically swaps in-memory state and
// flushes counters to disk. Legacy projects (created before this file
// existed) get a session synthesized on first open from their home HTML
// archetype (see projects::read_session).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Landing,
    App,
    Ambiguous,
}

impl Default for Mode {
    fn default() -> Self { Mode::Ambiguous }
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Landing   => "landing",
            Mode::App       => "app",
            Mode::Ambiguous => "ambiguous",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "landing" => Some(Mode::Landing),
            "app"     => Some(Mode::App),
            "ambiguous" | "?" | "" => Some(Mode::Ambiguous),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub mode:          Mode,
    /// The idea/brief the user seeded the project with.
    #[serde(default)]
    pub brief:         String,
    /// Currently active page slug within a multi-page project (usually "home").
    #[serde(default = "default_home")]
    pub active_page:   String,
    /// Archetypes already tried on this project (for the "Try different
    /// layout" flow — persisted so a restart doesn't repeat them).
    #[serde(default)]
    pub tried_archetypes: Vec<String>,
    /// Cumulative token usage for this project since it was created.
    #[serde(default)]
    pub tokens_in:     u64,
    #[serde(default)]
    pub tokens_out:    u64,
    #[serde(default)]
    pub created_at:    i64,
    #[serde(default)]
    pub updated_at:    i64,
}

fn default_home() -> String { "home".to_string() }

impl Default for Session {
    fn default() -> Self {
        let now = now_unix();
        Self {
            mode:            Mode::Ambiguous,
            brief:           String::new(),
            active_page:     default_home(),
            tried_archetypes: Vec::new(),
            tokens_in:       0,
            tokens_out:      0,
            created_at:      now,
            updated_at:      now,
        }
    }
}

impl Session {
    pub fn touch(&mut self) { self.updated_at = now_unix(); }
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Classify an idea into a Mode. Landing/marketing phrases → Landing.
/// App/dashboard/tool phrases → App. Neither matches → Ambiguous, and
/// the UI asks the user to pick.
pub fn infer_mode(idea: &str) -> Mode {
    // Ported from the JS landing regex in ui.html (pre-React), extended
    // with app-side keywords. Order matters slightly — landing patterns
    // win over app patterns for overlapping words like "app landing page".
    let text = idea.to_ascii_lowercase();

    // Landing / marketing indicators — direct wins.
    let landing_words = [
        "landing page", "landing", "marketing site", "marketing page",
        "homepage", "home page", "saas site", "product page", "launch page",
        "hero page", "sales page", "splash page", "coming soon",
        "waitlist page", "portfolio site", "personal site", "one-pager",
        "one pager", "single page",
    ];
    for w in landing_words {
        if text.contains(w) { return Mode::Landing; }
    }

    // App / dashboard / tool indicators.
    let app_words = [
        "dashboard", "admin panel", "admin ", "control panel", "workspace",
        "console", "portal", "internal tool", "crm", "erp", "saas app",
        "saas dashboard", "saas tool", "with login", "with signup",
        "with auth", "multi-page", "multiple pages", "multi page",
        "app ui", "app for", "product ui", "web app", "tracker",
        "analytics tool", "analytics dashboard", "monitor", "inbox",
        "kanban", "editor", "studio app", "logged-in",
    ];
    for w in app_words {
        if text.contains(w) { return Mode::App; }
    }

    // Standalone "app" is only decisive if there's no landing context —
    // "app landing page" would have matched above. Also treat "site"
    // alone as landing (marketing default).
    if text.contains(" app") || text.starts_with("app ") { return Mode::App; }
    if text.contains(" site") || text.starts_with("site ") { return Mode::Landing; }

    Mode::Ambiguous
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landing_wins() {
        assert_eq!(infer_mode("landing page for a scheduling app"), Mode::Landing);
        assert_eq!(infer_mode("marketing site for a coffee shop"),  Mode::Landing);
        assert_eq!(infer_mode("portfolio site"),                    Mode::Landing);
    }
    #[test]
    fn app_wins() {
        assert_eq!(infer_mode("dashboard for a fitness tracking app"), Mode::App);
        assert_eq!(infer_mode("admin panel for orders"),               Mode::App);
        assert_eq!(infer_mode("crm for freelancers with login"),       Mode::App);
    }
    #[test]
    fn ambiguous_falls_through() {
        assert_eq!(infer_mode("something creative"), Mode::Ambiguous);
        assert_eq!(infer_mode("coffee shop"),        Mode::Ambiguous);
    }
}
