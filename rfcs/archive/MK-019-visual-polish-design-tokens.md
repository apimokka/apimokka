# RFC MK-019 — Visual polish and design tokens

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Visual design / look-and-feel.
**Touches.** `theme.rs`, `widgets/mod.rs`, every screen and shell module.

## Why this RFC exists

v0.1–v0.3 focused on building the right *structure* — screens, message
flow, information architecture. The visual layer was an afterthought:
every screen invented its own font sizes (10/11/12/13/14/15/16/18/20/22/28),
padding (3/4/6/8/10/12/16/20/24), and border treatments (sometimes a hard
grey border, sometimes none, sometimes a different colour).

The result: a UI that worked but looked busy and inconsistent. Tight
spacing made everything feel cramped; heavy borders fragmented related
content; no clear type hierarchy meant the eye had no anchor.

## Design system

A `theme.rs` token vocabulary that every screen references:

### Spacing scale

| Token | Pixels | Use |
|---|---|---|
| `space::XS`  | 4  | Glyph + text gap, tight inline rows |
| `space::SM`  | 8  | Default gap between adjacent controls |
| `space::MD`  | 12 | Default gap between form fields |
| `space::LG`  | 16 | Gap between sections inside a card |
| `space::XL`  | 24 | Gap between cards / major regions |
| `space::XXL` | 36 | Page-level breathing room |

### Typography scale (six steps, all `f32` for iced 0.14)

| Token | Pixels | Use |
|---|---|---|
| `size::CAPTION`  | 11 | Tertiary metadata: timestamps, hints |
| `size::BODY_SM`  | 13 | Secondary content: list rows, button labels |
| `size::BODY`     | 14 | Default body text |
| `size::SECTION`  | 16 | Section headers ("WHEN — request matches") |
| `size::HEADING`  | 22 | Page headings |
| `size::HERO`     | 32 | Welcome hero only |

### Border radius

| Token | Pixels | Use |
|---|---|---|
| `radius::SM`   | 4   | Subtle (inputs) |
| `radius::MD`   | 8   | Cards (default) |
| `radius::LG`   | 12  | Larger surfaces |
| `radius::PILL` | 999 | Chips |

### Padding presets

| Token | Pixels `[v, h]` | Use |
|---|---|---|
| `pad::BUTTON`          | `[6, 14]`  | Default button |
| `pad::BUTTON_PRIMARY`  | `[10, 22]` | Hero / primary action |
| `pad::CARD`            | `[16, 18]` | Card interior |
| `pad::CHIP`            | `[4, 10]`  | Status chip |

## Style helpers

Replaced bordered card style with elevation-based:

- **`card_style`** — no border, subtle drop shadow (`alpha=0.04, blur=3px`)
  reads as a tangible thing without the visual noise of a grey rectangle.
- **`card_selected_style`** — primary-coloured 10% tint background + a
  primary-tinted shadow. The colour is supplementary; the elevation
  shift is what reads as "selected" (ABDD).
- **`chip_style`** — pill-shape with subtle background.
- **`panel_style`** — borderless sidebar tint (no hard rule).
- **`header_style`** — slight off-base background + a 2px-blur shadow
  for the top bar, providing visual separation from content without
  drawing a hard line.
- **`hairline_style`** — replaces the divider widget's background;
  40% alpha instead of solid grey, far quieter.
- **`muted_text(theme)`** — secondary-importance text colour (60% alpha
  on the base text colour). Used for hints, timestamps, captions.

## Per-screen polish

### Welcome
- Hero text bumped to 32px (`size::HERO`)
- Generous breathing room: 36px around hero, 24px between sections
- Recent-workspace cards larger (220px wide, more vertical space, dual-line metadata)
- Three-layer diagram chips bigger (260px wide, 22px glyph)
- Whole page constrained to 720px max-width and centered

### Routes left sidebar
- 220px → 260px wide for less cramping
- Rule-set rows: 14px padding, dirty dot moved to the right
- Indented rule rows: 12px indent space (`space::MD`)
- "+ Add rule" sub-button under each rule set
- Section headers (Rule sets, Fallback, Middleware) use 11px caption
- Subtle dividers between sections

### Rule builder (the most-used surface)
- Card section header at 14px body (vs prior 13px)
- Section padding: 16/18 (vs prior 10px)
- Tab buttons (Inline/Serve file) use selected card style for active state
- URL operator pick-list: 140px wide (vs prior 120)
- Test Rule button is now `pad::BUTTON_PRIMARY` (10/22)
- WHEN → RESPOND arrow at 22px (`size::HEADING`)

### Trace strip
- 260px → 280px wide
- Each event row is a card with 8/12 padding
- Two-line layout: method+path on top, timestamp+duration on bottom in muted text
- Outcome glyph at 14px (vs prior 12)
- Replay button outside the card so it's reachable without selecting the row

### Top bar
- 12/24 padding (vs prior 8/16) — more breathing room
- Workspace identity uses muted "·" separator (no longer plain text)
- Status chips (server/save) use `chip_style` pill shape
- View controls (trace toggle, palette, locale) visually grouped with extra 12px gap before them
- Replaced `chrome_container_style` with `header_style` for the subtle shadow

### Right inspector
- 200px → 260px wide
- Section header at 16px (`size::SECTION`)
- Quick-action buttons stack vertically, full-width
- Validation badges padded properly
- Empty state for "no rule selected"

### Dialogs (Confirm, Test Rule, Command Palette)
- 24px page padding (vs prior 16/20px mix)
- Confirm dialog: 420px wide, 22px title, muted-text description
- Test-rule dialog: 520px wide
- Command palette: 520px wide, "Esc" hint in the header

### Wizard
- 680px × 620px container (vs prior 640 × 580)
- 24/32 page padding
- Section cards use `pad::CARD` consistently
- Sticky action bar uses `header_style` background for visual separation

## Bulk size migration

A Python pass also migrated remaining screens (settings, trace,
match_detail, scripts, dashboard, dotted_path, bottom_drawer) from
hardcoded sizes to the token scale:

- `.size(10)`/`.size(11)` → `size::CAPTION`
- `.size(12)`/`.size(13)` → `size::BODY_SM`
- `.size(14)`/`.size(15)` → `size::BODY`
- `.size(16)`/`.size(18)` → `size::SECTION`
- `.size(20)`/`.size(22)`/`.size(24)` → `size::HEADING`

This eliminates the prior 12-size salad in favour of six clear steps.

## Cumulative impact

Before this RFC: ~110 distinct font-size + padding magic numbers
scattered across 24 source files.

After: 6 size tokens, 6 spacing tokens, 4 padding presets, 4 radii.
Every screen draws from the same vocabulary. Future visual tweaks
happen in one place.

## Out of scope

- Dark theme (Theme::Dark works mechanically but selected-card
  primary-tint may need tuning)
- Animation / transitions (iced 0.14 supports them; deferred)
- Custom button styles (using iced defaults for now; a `button_style`
  helper is a v0.5 candidate)
- Density toggle (compact / comfortable)
