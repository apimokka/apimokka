# RFC MK-017 — Packaging, performance, and release gates

**Status.** Superseded by RFCs MK-021–MK-037 — workflow-centred redesign series
**Tracks.** Release artifacts, performance benchmarks, review checklist.
**Touches.** Workspace Cargo manifest, `README.md`, `CHANGELOG.md`, release archive

## Implementation summary

| Acceptance criterion | Met? | Notes |
|---|---|---|
| P0 RFCs have reviewed acceptance status before release | ❌ | This audit (`AUDIT-2026-05-22.md`) is the first systematic review |
| App remains lightweight enough to match project positioning | ⚠️ Untested | No benchmarks; release archive is 88 KB source (binary size not measured) |
| Trace memory growth is bounded | ❌ | `App::trace: Vec<MatchTraceEvent>` is unbounded; production needs a ring buffer |
| Release artifacts include documentation and known limitations | ✅ | README §"Known limitations" + CHANGELOG + this RFC + AUDIT doc |

## What was done

- Cargo workspace structure with three crates (model / i18n / app)
- `rust-version = "1.91"`, `resolver = "3"`, edition 2024
- Apache-2.0 LICENSE + NOTICE files
- README with badges, hero, quick-start, design notes
- CHANGELOG noting v0.1.0 additions and known limitations
- Release archive `apimokka-mockup-v0.1.0.tar.gz` (88 KB, excludes target/)
- `docs/src/` mdbook skeleton (README + architecture)

## What was not done

- No `cargo bench` benchmarks
- No binary size measurement after release build (`cargo build --release` succeeded but artifact size not captured)
- No automated review checklist (`docs/supplemental/mockup-review-checklist.md` from the source RFC package was not adapted)
- Trace buffer is unbounded — would OOM under sustained traffic

## Recommendation

Before any v0.2.0 cut: add the AUDIT pattern as part of the release
process. Each release should ship an audit doc enumerating per-RFC
status against acceptance criteria, not a marketing-tone changelog.
