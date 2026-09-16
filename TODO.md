# TODO — Feast Frenzy

## Standards alignment

- [ ] Expose game logic through `src/lib.rs` and use it from `main.rs`; migrate all ten `src/**/tests.rs` suites and their helpers into `tests/`, removing source-level test modules before expanding coverage (§11.4). Preserve regressions and consolidate related data-integrity cases toward five per feature, documenting justified exceptions (§11.3).
- [ ] Use toolkit typed loaders in the data-integrity tests instead of direct `serde_json::from_str` calls. Add startup semantic validation for IDs, references, effect keys, and balance invariants in `GameData::load`, with clear diagnostics instead of silently accepting invalid content or empty defaults (§5.3, §6).
- [ ] Move hardcoded gameplay probabilities, movement speeds, timing, and balance multipliers from `engine.rs`, `player.rs`, and `state/cinematic.rs` into validated JSON configuration (§5.3).
- [ ] Move player-facing Rust literals (UI labels, tooltips, gameplay messages, fallback chatter, and goal templates) into JSON under `assets/`, loaded through the toolkit (§5.3).
- [ ] Fix `ui/layout.rs` minimum widths/heights that cause overlapping panels below 972px wide or off-screen content in short windows; add responsive layout or intentional virtual-resolution scaling, including usable modal and touch targets (§7.5).
- [ ] Make control guidance touch-first in `README.md`, `game_page.json`, tutorial data, and UI hints: name visible Cook, Serve, VIP, Clear Carried Dish, and prestige controls alongside shortcuts. Make hover-only meter/guest explanations accessible by tap (§7.5).
- [ ] Replace the blanket mouse-down activation in `commands.rs` with shared toolkit release-based input handling; preserve modal ownership and document any intentional press-only actions (§7.4).
- [ ] Split oversized functions such as `gameplay::serve_customer`, `commands::apply_ui_command`, and `ui::layout::draw_top_header` into cohesive helpers under 100 lines. Group related arguments and remove the crate-wide `too_many_arguments` allowance; explain any remaining targeted allowances (§4, §10.2).
- [ ] Add missing module-purpose `//!` comments in app, data, state, gameplay, input, and UI modules; correct the source-gate test comment to describe total physical lines, including tests (§9.2, §2.2).

## Simulation and shared effects

- [ ] After test migration, add deterministic simulation coverage for day-summary pause/resume, event eligibility/effects/expiry, and returning-regular identity/progression across visits. Inject controllable randomness where needed; retain existing day-cycle and guest unit coverage.
- [ ] Make screenshot timing deterministic: pass the capture harness timestep into `App::tick` and derive ambient animation timing from the supplied clock instead of wall time (`app.rs`, `ui/ambience.rs`).
- [ ] Adopt the existing toolkit `fx::FloatingTextLayer` for shared floater lifetime/cap/fade behavior, preserving Feast Frenzy's floor/header anchors and gain styling (`state/floaters.rs`, `ui/floaters.rs`).

## Content and presentation

- [ ] Expand customer types into a fifth tier, including unlock costs, meat/recipe links, portraits, and content validation.
- [ ] Add dining events and prestige perks beyond the four entries in each JSON catalog, with validated effects and descriptions.
- [ ] Add varied, data-driven next-day goals beyond clientele unlocks and prestige; move goal selection out of `ui/day_summary.rs` into game logic.
- [ ] Add decor and room variants tied to clientele progression, extending the existing tone tint into visible room changes.
