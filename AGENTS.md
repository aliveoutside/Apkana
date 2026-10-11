# AGENTS.md — Apkana

Apkana is a Rust desktop GUI using `eframe/egui 0.32`.

UI surfaces: Decode + Build, Sign, Merge, Install, tool settings, and resizable output logs.

Run `cargo check` and `cargo test` after changes. Use `cargo fmt` before committing.

Architecture:
- `src/main.rs`: eframe native window entrypoint
- `src/app.rs`: application state, background task channel, dispatch
- `src/app/view.rs`: egui panels/navigation, output, settings
- `src/ui/*_tab.rs`: tab-specific egui widgets and input messages
- `src/app/handlers.rs`: validated workflows and settings updates
- `src/app/pipeline.rs`: build → align → sign → install sequencing
- `src/app/validation.rs`: validation
- `src/config.rs`: persisted settings
- `src/tools/*`: external tools and native split-package merger

Background tasks must never mutate egui app state off-thread. Use `Task::perform` and
the channel to deliver completion messages to the UI thread.

Preserve merger behavior: filter only signature-related META-INF entries, retain base
resources.arsc on ARSC conversion failure, and report OBB files without merging them.
Do not update the pinned resand revision without merger-focused verification.
