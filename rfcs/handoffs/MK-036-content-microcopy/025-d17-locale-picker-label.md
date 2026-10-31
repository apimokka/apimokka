# Dev-team task 025 — the locale picker is labelled "Settings" (D-17)

**Issued:** 2026-10-31
**Issued by:** programme architect
**Authorized by:** project owner — pending
**Defect:** D-17 (S2)
**Independent of D-12's scope decision** — this can proceed whatever is decided about task 024.

---

## 1. Purpose

Label the language control with its own name. Today the control that changes the
interface language is labelled **"Settings"**, inside the Settings screen, in both
locales.

## 2. Background

`crates/app/src/screens/settings.rs:90`:

```rust
widgets::field(
    app.t(Key::NavSettings),
    pick_list(Locale::all().to_vec(), Some(app.locale), Message::ChangeLocale)
    …
)
```

`Key::NavSettings` is the **navigation** label — `"Settings"` in `en.rs:53`,
`"設定"` in `ja.rs:50`. It is correct where it is used for navigation and wrong
here.

Visible in every Settings frame of task 023's capture
(`.git-exclude/release-evidence/2026-10-31-task023-d15-text-input-capture/`) and
confirmed at source.

**Why S2 rather than cosmetic:** MK-056's scenario 6 is *"change theme and locale,
and switch audience mode"*, so a participant will look for this control. One
hunting for language finds a dropdown labelled "Settings" showing "EN". The label
names the wrong thing — objective, not a matter of taste.

## 3. Required implementation

1. **Add one key** — a language/locale field label — to `apimokka_i18n::Key`, with
   values in **both** `en.rs` and `ja.rs`. Neither exists today: the only related
   keys are `PaletteCmdLocale` ("Change locale") and the `LocaleEn`/`LocaleJa`
   values.
2. Use it at `settings.rs:90` in place of `Key::NavSettings`.

**Wording is yours**, within two constraints: it should name the thing being
chosen rather than the act of choosing (it is a field label, not a command, so
"Change locale" is the wrong register), and the Japanese must be a real
translation rather than a transliteration of the English.

## 4. Non-change scope

- **Do not touch `Key::NavSettings` itself** — it is correct at its navigation call
  sites, and this fix must not disturb them.
- **No change to `PaletteCmdLocale`**, to the `LocaleEn`/`LocaleJa` values, or to
  any other label.
- No change to the picker's behaviour, width, or position.
- **Do not add a palette row for locale.** That is task 024's territory and is
  blocked on the owner's scope decision.
- No `theme.rs` change. D-15 is closed.

## 5. Required tests

Whatever the i18n crate's existing convention is for asserting a key resolves in
both locales — follow it rather than inventing a shape. If the convention does not
cover new keys, say so; a plain statement is better than a test that asserts the
constant equals itself.

## 6. Required evidence

- `bash scripts/check-release-gates.sh` green on both toolchains.
- **One Settings frame per locale** — EN and JA — showing the corrected label.
  Theme does not matter; one preset is enough, because this is a content fix and
  the contrast work is closed and asserted. Use task 022's nested-compositor recipe
  with the coordinate conversion; no owner clicks needed.
- Record backend, renderer and build SHA.

**This is a case where the rules' "evidence answers a question" test matters:** the
question is *does the label read correctly in both locales*, so two frames answer
it and twenty would not answer it better.

## 7. Acceptance criteria

1. A new key exists with values in both locales, and `settings.rs:90` uses it.
2. `Key::NavSettings` unchanged and still correct at its own call sites.
3. Gate green.
4. Two frames, EN and JA, with configuration recorded.

## 8. Known risks

- **Japanese register.** The i18n file is the specification a production team will
  translate from; a loose Japanese label propagates. If unsure of the right term,
  say so in the submission rather than guessing silently.
- **Scope creep into the locale palette row.** See §4.

## 9. Required review-request content

Per workflow §9.2: the key added and both values; the call site; tests or a plain
statement of why none discriminates; gate status; the two frames with their
configuration; anything about the Japanese wording you want a second opinion on.

Commit, and **do not push without authorization**. Check
`git log origin/mockup-v0-01..HEAD` first and say what a push would carry. Commit
messages: digest plus optional body within 400 characters.
