# lucid design

**A native macOS app that turns a one-line idea into a polished landing page.**

Describe your product. Pick a style. Watch it assemble section by section. Swap any section with one click — zero AI tokens, zero wait. Export clean, self-contained HTML.

> Built in Rust · WKWebView · No Electron · No web server · One binary

---

## Install

```bash
cargo install lucid-design
```

Requires macOS 11+, Rust ([rustup.rs](https://rustup.rs)), and Xcode CLI tools.

```bash
xcode-select --install  # if you haven't already
```

First install compiles from source (~2 min). After that, just run:

```bash
lucid-design
```

---

## Setup

Create `~/.config/lucid-design/config.toml` with your preferred LLM:

**Claude Code (no API key needed)**
```toml
provider = "claudecode"
model    = ""
api_key  = ""
```

**Anthropic API**
```toml
provider = "anthropic"
model    = "claude-sonnet-4-6"
api_key  = "sk-ant-..."
```

**Other supported providers:** `openai`, `groq`, `ollama`, `gemini`

---

## How it works

**1. Describe your idea**

```
a project management tool for indie game studios
```

**2. Pick your kit** — or leave everything on Auto

Choose from chips for Theme, Palette, Navbar, Hero, Features, Testimonials, Pricing, CTA, and Footer. One LLM call fills all the copy. Sections stream in top-down as they're assembled.

**3. Swap sections without spending tokens**

Click any section on the canvas → pick an alternate variant from the panel → it swaps instantly. No LLM call. No wait. Undo with Cmd+Z.

**4. Refine via chat**

Type anything to edit the whole design. Click an element first to scope the change to just that part.

**5. Export**

One click → self-contained HTML file in `~/Documents/lucid-design/`. Open in any browser.

---

## What's in the library

17 hand-authored HTML sections across 7 categories, each using CSS custom properties so palette swaps re-tone everything automatically:

| Category | Variants |
|---|---|
| Navbar | Brand-heavy serif · Sticky transparent · Centered editorial |
| Hero | Centered editorial · Split product shot · Bento with stats |
| Features | Alternating rows · Bento varied · Icon triad |
| Testimonials | Three-card grid · Hero quote |
| Pricing | Three-tier · Single plan |
| CTA | Centered band · Split with form |
| Footer | Minimal hairline · Giant wordmark |

**12 themes** — editorial warm cream, saas minimal white, dark refined, corporate blue, luxury fashion, brutalist mono, cyber neon, and more.

**10 palettes** — each a set of CSS custom properties that layer on top of any theme.

---

## Token cost

| Action | Cost |
|---|---|
| Generate from kit | ~500 output tokens (copy-fill only) |
| Swap a section | **0 tokens** |
| Refine via chat | ~2k tokens |
| Full freeform generation | ~10k tokens |

---

## Projects are saved automatically

```
~/Documents/lucid-design/
├── my-app.html        ← the design, autosaved on every change
├── my-app.chat.json   ← full conversation, restored on reopen
└── ...
```

---

## Contributing

Add a new variant by appending a block to `src/ai/variants.md` and adding it to `KIT_VARIANTS` in `src/assets/ui.html`. Rebuild and it's live.

```bash
git clone https://github.com/ankush109/lucid-design
cd lucid-design
cargo run
```

---

## License

MIT OR Apache-2.0
