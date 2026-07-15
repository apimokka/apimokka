# RFC MK-007 — Visual rule builder screens

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Routes screen / rule editing UX.
**Touches.** `crates/apimokka-app/src/screens/rule_builder.rs`, `screens/routes.rs`

## Summary

The rule builder (S-08) renders the selected rule as five stacked card sections:

1. **URL path** — enable checkbox + text input + operator pick-list.
   The operator pick-list is disabled when the path field is empty
   (RFC-013 validation: `url_path: None` + `url_path_op: Some(_)` is invalid).

2. **Method** — segmented control: ANY / GET / POST / PUT / DELETE / PATCH.
   "ANY" maps to an empty method string.

3. **Headers** — per-condition rows: name + operator pick-list + value input.
   Value input is hidden when operator is `Exists` or `Absent`
   (via `HeaderOp::value_irrelevant()`).

4. **Body conditions** — per-condition rows: dotted-path input + "…" assistant
   button + operator pick-list + value input.
   Rows with a `$.`-prefixed path show an inline JSONPath warning
   (`Key::DottedPathJsonpathWarn`) to educate users about the dotted-path
   syntax difference.
   Value input is hidden for `Exists`/`Absent`.

5. **Respond** — inline-text / serve-file tab switch + body editor + status
   + delay fields.

## Dotted-path assistant (S-09)

Rendered as a snora `Dialog`. Accepts raw JSON in a textarea, extracts leaf
paths via a naive recursive parser (mockup quality — handles the canned
`PathAssistantState` JSON correctly), displays a clickable path tree, and
inserts the selected path back into the body-condition row that triggered it.

## Body operator value input shape

| Operator category | Input control |
|---|---|
| String coercion, type-aware, regex | Text input |
| Numeric | Text input (validated at save time in production) |
| `Exists`, `Absent` | Hidden |
| `ArrayContains`, `EqualTyped` | Text input (JSON-literal) |

Full shape-switching is in scope for a future release; this mockup uses a
uniform text input for simplicity.
