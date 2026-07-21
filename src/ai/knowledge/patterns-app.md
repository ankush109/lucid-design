# Design Knowledge — App / Dashboard Patterns

Dashboard, admin panel, and product-UI shell/widget patterns. Load only for APP mode.

---

## 15. Dashboards and product UIs — beyond the landing page

A dashboard is NOT a landing page. It has no hero, no CTA, no pricing tiers. It has a **shell** (sidebar/topbar), a **workspace** (the main area), and **widgets** (things that display or act on data). Get the shell right and everything else falls into place.

### 15.1 Shell archetypes

Pick ONE — do not combine two shell types in a single design.

**Sidebar shell** (Linear, Notion, Vercel dashboard, Stripe, Retool)
- Fixed 240–280px left rail, dark or tinted-neutral.
- Rail sections: workspace switcher (top), primary nav (middle), user/settings (bottom).
- Primary nav items: 6–10 max, grouped with dividers or headers, each with a 16–20px icon + label.
- Active state: subtle left border accent + slight background tint, never a full pill.
- Collapsible on desktop (rail → icon-only), hidden on mobile behind a menu.

**Topbar shell** (Attio, Airtable, Figma)
- 56–64px top nav, workspace switcher on the left, search center, user/notifications right.
- Secondary nav below (tabs) OR left rail below topbar (mixed shell).
- Works when horizontal breadth matters (spreadsheet-like tools, canvases).

**Mixed shell** (Linear Insights, Datadog, Notion databases)
- Topbar (48–56px) with workspace and account.
- Left rail (200–240px) for section navigation.
- Optional right rail (280–360px) for context / filters / details.
- Highest information density. Best for analytics + data tools.

**Rules for all shells:**
- Shell is present on every page of the app — same rail, same topbar, only workspace content changes.
- Nav labels are concrete nouns from the product's domain, not "Dashboard / Overview / Analytics" everywhere.
- Icon set is consistent (all outline, or all filled — never mixed) at consistent stroke weight.

### 15.2 Workspace patterns

The main content area follows a small set of patterns. Match to the data type.

**Grid of stat cards** — 3 to 6 KPI tiles across the top, showing single scalar values with delta and mini-trend. Below: a chart or table. Best for executive views.

**Chart-first** — one large chart or map dominates, controls and filters above/beside, related tables below. Best for analytics and observability.

**Table-first** — a data table fills the workspace, with filter bar above, row detail in a right rail or drawer. Best for CRUD tools (users, orders, tickets).

**Split canvas** — left column: list / navigation of items. Right column: detail of the selected item. Best for inbox-like tools (issues, threads, records).

**Editor canvas** — full-bleed canvas or textarea with floating toolbars, right rail for properties. Best for editors (Notion, Figma, Linear issue view).

**Card feed** — vertical list of cards with rich content. Best for social feeds, activity streams, notification centers.

### 15.3 Common widgets

**Stat card**
- 1 metric name (12px, muted, uppercase or normal case), 1 large number (32–40px, tabular numerals), 1 delta indicator with arrow + percentage + color (green up / red down / grey neutral), 1 optional sparkline/mini-chart below.
- 3–6 cards in a row on desktop, 1–2 per row on mobile.
- Padding: 20–24px inside; gap 16–20px between.
- No border chrome — use tinted-neutral background against paper.

**Data table**
- Sticky header row, subtle bottom-border rows (never full grid lines).
- Cell padding 12–16px vertical, 16–20px horizontal.
- Right-aligned numeric columns, tabular numerals.
- First column often bolder or wider (the "name" column).
- Row hover: slight tint of accent. Row click → open detail.
- Column headers sortable; show sort indicator.
- Empty state: illustration + one line + primary action, NOT a blank grid.
- Pagination or infinite scroll — pick one.
- Filter bar above: search + 2–4 filter chips + view switcher.

**Chart widget**
- Chart type matches data: line for time-series, bar for categories, donut only for 2–4 slices (never more), area for cumulative.
- Y-axis often unlabeled if number is self-evident. X-axis: dates in short form (Mon 12, Jan 3).
- No gridlines behind bars/lines unless data density demands it — one horizontal reference line at the baseline is enough.
- Legend inline with chart title, not as a separate block.
- Interactive tooltip on hover: date + value + delta from previous.
- Empty state: same rule as tables.

**Activity feed / timeline**
- Left-aligned timeline dot + line, right-aligned event copy.
- Group by day: "Today", "Yesterday", specific date.
- Each event: actor + verb + object + relative time ("Ankush closed issue #42 · 2h ago").
- Avatar or icon for the actor.

**Filter bar**
- Search input (with keyboard shortcut hint on the right — `⌘K`).
- 2–4 filter chips (each opens a popover): Status, Owner, Date range, Tag.
- View switcher on the right (list / grid / kanban).
- Sort dropdown after view switcher.
- Never crowd — if you need 6+ filters, use a slide-out filter panel triggered by an "All filters" button.

**Detail drawer / right rail**
- 360–480px wide, slides in from the right when a row/item is clicked.
- Header: item name, close button (top-right), primary actions (top-right or bottom-right).
- Body: property list (label:value pairs, label left aligned, value right aligned) then rich content sections.
- Footer: secondary actions or metadata (created / updated timestamps).

**Command palette (⌘K)**
- 480px centered modal overlay with 30% dim behind.
- Input at top, results below grouped by type (Actions / Navigation / Documents).
- Each result: icon + label + shortcut hint on right + secondary text.
- Keyboard-first — arrow keys navigate, enter runs, esc closes.

**Empty states**
- Every list, table, chart, panel needs one.
- Structure: illustration or icon (small, not a giant graphic) + one-line explanation + primary action button.
- Example: "No issues assigned to you yet. [Create issue]"
- Never blank space. Never "No data."

**Notification / toast**
- Bottom-right or top-right corner, 320–400px wide.
- Icon (info / success / warning / error, color-matched), title, optional detail line, close button, optional action link.
- Auto-dismiss after 4–6s for info/success, sticky for warning/error.

### 15.4 Density and hierarchy

Dashboards live at a different density than marketing pages. Get the math right:

- Base spacing unit: 4px (dashboards are denser than the 8px used for landing pages).
- Card padding: 16–24px (not 32–48px).
- Section spacing: 24–32px between panels (not 96–120px).
- Body text: 14px (not 16–18px).
- Line-height: 1.4–1.5 (not 1.6).
- Font weight jumps are smaller: 400 body → 500 emphasis → 600 headings (not 400 → 700).

Hierarchy without size:
- Muted text (60–70% of ink color) for labels.
- Tabular numerals for all numbers.
- Icons scale with text (never bigger than the accompanying label's cap-height + 20%).

### 15.5 The dashboard color system

- Paper (workspace background): near-white, warm off-white, or near-black.
- Rail (sidebar): tinted-neutral, slightly darker/lighter than paper. Never pure grey.
- Card surface: paper, or one step of elevation from paper (2–3% shift, not a shadow).
- Ink: main text, 90–95% of pure black or pure white.
- Muted ink: labels, timestamps, secondary text. 55–65% of ink.
- Accent: brand color, used ONLY for interactive elements and the ONE key data callout.
- Semantic: green (positive delta), red (negative delta), amber (warning). Distinct from accent.
- Chart colors: a set of 5–7 hues designed to sit on card surface — NOT the accent color.

### 15.6 When the freeform LLM path is asked for a dashboard

1. Choose a shell archetype (usually sidebar or mixed for admin tools).
2. Choose the primary workspace pattern (stat grid + chart + table is the most common).
3. Realistic domain nav labels ("Athletes / Workouts / Programs / Payments" for fitness, not "Users / Analytics / Reports").
4. Real data in every widget — never Lorem Ipsum, never placeholder numbers like "10K" or "99.9%". Use specific, believable, odd numbers: "1,847 active this week", "$28,412.53 pending payout".
5. Include at least one empty state somewhere so the design shows how it degrades.
6. Every nav item is a real `<a>` — do NOT use `href="#"`. Use `href="./settings.html"`, `href="./users.html"` — real filenames so the multi-page navigation flow can wire them up later.

### 15.7 Multi-page apps (roadmap concept — currently designs are single-page)

Real product UIs span multiple pages: Home → Settings → Users → Reports. Design intent:

- Design page 1 (Home) with a working nav pointing to real filenames.
- User clicks an unwired nav link on canvas → prompt: "Design the {settings} page?"
- If yes → new sub-design in the same project, inheriting the shell (sidebar, topbar, theme).
- Tab bar above canvas lets user switch between pages.
- Export packages all pages as a folder of linked HTML files.

Until multi-page ships, dashboards SHOULD use real relative-path filenames in nav (`./settings.html`, etc.) so that when multi-page ships, existing designs upgrade cleanly.

---

