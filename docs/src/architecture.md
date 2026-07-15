# Architecture

## Crate dependency graph

```
apimokka-app  ──depends──► apimokka-model
     │                     apimokka-i18n
     │                     iced 0.14
     │                     snora 0.25
     │
apimokka-i18n ──depends──► (nothing — no external dependencies)
apimokka-model ─depends──► uuid
```

## App state machine

```
AppView::Welcome ──OpenWorkspace──► AppView::Workspace
                 ──WizardStart───► AppView::Wizard ──WizardCreate──► AppView::Workspace
                 ──GoDashboard──► AppView::Dashboard ──OpenWorkspace──► AppView::Workspace
```

## Shell composition (snora AppLayout)

```
AppLayout::new(shell_body)
    .header(top_bar::view)
    .side_bar(zero_width_placeholder)   // rail is in shell_body row
    .sheet(bottom_drawer)               // when drawer.is_some()
    .dialog(command_palette | path_assistant)
    .on_close_modals(close_msg)
```

The left rail is embedded in the `shell_body` row directly (not in the
AppLayout `side_bar` slot) to give full control over width, border, and
padding.

## Message flow

```
User interaction
      │
      ▼
iced calls App::update(msg)
      │
      ├── Mutates App fields (selection, wizard state, etc.)
      ├── On EditCommand-class messages: mutates snapshot,
      │   marks dirty, rebuilds validation
      ├── On Save: simulate_save() → builds SaveResult
      └── On StartStopServer: toggles ServerState

App::view() is called after every update
      │
      ├── AppView::Welcome → screens::welcome::view
      ├── AppView::Dashboard → screens::dashboard::view
      ├── AppView::Wizard → screens::wizard::view
      └── AppView::Workspace → shell::view::view
                                  │
                                  ├── top_bar, rail, screen body (tab dispatch)
                                  ├── right inspector (Routes only)
                                  ├── sheet (drawer)
                                  └── dialog (palette / path assistant)
```

## i18n architecture

`apimokka-i18n` defines a flat `Key` enum (~200 variants). Each locale
module (`en.rs`, `ja.rs`) implements a `match` expression covering every
variant — missing arms are compile errors. `Tr { locale }` wraps the
lookup:

```rust
tr.t(Key::BtnSave)  // returns &'static str
```

Locale is stored in `App::locale` and changed via `Message::ChangeLocale`.
`Locale` implements `Display` for use in iced `pick_list`.

## File line-count targets

Per project guidelines:
- Aim for < 300 ELOC per `.rs` file
- Split at > 500 ELOC

Current counts (approximate):
- `app.rs`: ~450 (acceptable for central reducer)
- `screens/rule_builder.rs`: ~240
- `screens/settings.rs`: ~190
- `shell/bottom_drawer.rs`: ~170
- All other screens: < 150
