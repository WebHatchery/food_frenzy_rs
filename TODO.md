# TODO — Feast Frenzy

## UI_STYLE review — 2026-09-20

Audit and planning only; no UI implementation in this change. The previous file
contained only “No remaining AI-actionable tasks”; there were no task checkboxes
or completion records to merge. Retain the existing tracked `TODO.md` filename.

### Evidence and scope

- Read `AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`,
  `GAME_DEVELOPMENT_GUIDE.md`, and `README.md`. The local UI style guide matches
  `../rust_management/docs/UI_STYLE.md`. No GDD or `PROJECT_AGENTS.md` was found.
  The README has no screen briefs or declared minimum viewport. Preserve its
  explicit cozy marketing/editorial stance when adding technical screen plans.
- Inspected the screen dispatcher, command routing, all principal UI views,
  floor/table transforms, progression actions, tutorial and content catalogs.
  Current flow: Title -> New/Load -> Playing; Settings is reachable from Title.
  Playing layers the clientele board, specialization, prestige, closing ledger,
  and Lounge sequence over the same service layout.
- Visually inspected existing `docs/verification/ui_gameplay.png`,
  `ui_dining_rush.png`, `ui_clientele_board.png`, `ui_specialization.png`,
  `ui_day_summary.png`, `ui_lounge.png`, `ui_title.png`, and `ui_settings.png`.
  These are **1920x1061** images, not proof of a 1920x1080 canvas or current-build
  behavior. Some are stale: Settings lacks the sound row now drawn by code;
  older modal backgrounds show a different prestige requirement; tutorial copy
  has since changed. Screenshot findings below describe those saved images;
  code findings describe the inspected working tree.
- No fresh launch, capture, browser/touch interaction, or publish was performed
  for this documentation-only pass. Existing evidence is sufficient to plan
  the confirmed composition and code issues, but does not establish current
  visual correctness, high-DPI picking, or small-screen usability.
- The UI is game-specific; no literal template demo labels were established.
  The permanent dashboard/card composition still needs the subtraction and
  recomposition required by UI_STYLE §§2–4 and 8. Keep the existing contextual
  Serve/VIP controls, pin-able guest information, transient gain floaters,
  closing ledger, and Lounge reward focus as useful foundations.

### Verification contract for implementation tasks

Use **N = 1920x1080 actual canvas** as the proposed normal target, plus
**1280x720** desktop and **M = 844x390 actual canvas** as a proposed minimum
landscape touch target. These are future test targets, not supported-size claims.
Task UI-01 must ratify/document the support contract; adjust M explicitly if
product requirements differ. Also inspect 390x844 portrait for responsive play
or a readable orientation/support notice with reachable recovery controls.
Check the embedded browser canvas, not just browser window dimensions.

For each affected screen, capture normal and minimum sizes, exercise visible
controls without keyboard/hover, and inspect long labels, large values, and
dense/urgent states. Store new evidence directly in `docs/verification/`,
replacing equivalent captures. Record actual canvas size, build, input path,
and remaining limitations. After meaningful game changes run `./publish.ps1`
without parameters and report the outcome; screenshots alone are insufficient.
Keep rendering read-only, route intents through commands, use toolkit layout/
pointer helpers where applicable, and respect the 800-line Rust file limit.

### Verified findings — ordered by impact and dependencies

- [ ] **UI-01 — Recompose service around cooking and serving, with progression on demand.**
  **Screen/files:** Normal service and first shift; `README.md`,
  `src/ui/layout.rs::{layout_rects, draw_header_tiles, draw_and_collect_hitboxes}`,
  `src/ui/kitchen.rs::draw_kitchen`, `src/ui/growth.rs::draw_growth_panel`,
  `src/ui/common.rs`.
  **Observed:** Code always draws six resource tiles, a kitchen hero and four
  station cards, the floor, five progression cards, and an event footer.
  Gameplay/rush images confirm competing bordered regions, duplicate Guests
  headings, guest names repeated in the sidebar, repeated ready counts, and
  prestige repeated in the header and right panel. The floor occupies about
  55% of image width but its active guests are small and widely separated.
  **Change:** First write UI_STYLE §1 briefs for service, management, and modal
  decisions and declare supported viewports. Make the guest/order floor the
  dominant region and stations/carried dish the supporting region; retain a
  quiet, compact current-state strip. Replace the permanent growth column with
  a visible management entry and contextual availability cue. Move pantry,
  upgrades, crafting, and prestige details into that view. Remove the redundant
  guest list, duplicate ready totals/headings, nested decorative frames, and
  oversized decorative kitchen hero; preserve station freshness/capacity,
  urgent patience, action costs, and the Clear Carried Dish recovery control.
  Reveal Lounge economy and prestige details when relevant, with a discoverable
  route to explanations. Do not merely leave empty space where panels were.
  **Acceptance:** A player can identify the next Cook/Carry/Serve decision at
  a glance; normal service has at most 2–3 strongly competing attention regions.
  Management is easy to find but does not demand attention during every order.
  **Verify:** At N, 1280x720, and M compare fresh shift, carrying, rush, and
  advanced service; tap from service to management and back without losing the
  selected dish or access to urgent information. Foundation for UI-02–UI-04.

- [ ] **UI-02 — Replace shrinking columns with responsive service framing and usable targets.**
  **Screen/files:** Service across viewport sizes; `src/ui/layout.rs::layout_rects`,
  `src/ui/common.rs::{floor_to_screen, kitchen_to_screen}`, `src/ui/dining.rs`,
  `src/ui/dining/room.rs`, `src/ui/kitchen.rs::draw_recipe_station`,
  `src/engine.rs::restaurant_table_position`, `src/ui/actors.rs`.
  **Observed (code):** At width 972 the side-column rule jumps to a 320px minimum
  per side, leaving a 300px floor; immediately below it the floor is about 460px.
  At 1280 the floor is 608px wide. World coordinates stretch independently with
  floor width/height while tables (96px), guest sprites (68x80), the 220px Lounge,
  and labels remain fixed in screen pixels. The info hitbox is 22x22, table Serve
  is 50x24, VIP is 72x28, and several management controls are 24–30px high.
  Narrow/short layouts have fixed minimum content heights without scrolling.
  These are verified geometry constraints, not a claimed small-screen playtest.
  **Change:** Following UI-01, reflow stations and collapse secondary content
  at explicit breakpoints before shrinking text. Frame occupied/serviceable
  tables at a consistent useful scale, accounting for labels, inspectors, door,
  and Lounge reservations. Use a shared toolkit-supported transform for
  rendering/picking; if pan/zoom is needed, expose touch controls and a reset.
  Keep table identities/world positions stable. Give essential targets a
  proposed minimum 44x44 CSS-pixel equivalent and non-overlapping hit areas;
  kitchen rows already use larger row hitboxes, so measure actual hitboxes.
  **Acceptance:** Resize never abruptly compresses the floor or hides a core
  control; every supported guest is readable/selectable and touch targets remain
  usable after browser scaling. No negative-width cooking bars or overflow.
  **Verify:** N, 1280x720, M, 971/972/973px widths, portrait behavior, and display
  scaling; repeat Cook -> Carry -> Serve -> Clear and guest inspection after
  resize. Check starter tables and maximum upgraded capacity. Depends on UI-01.

- [ ] **UI-03 — Give each guest one readable order and one unobscured service action.**
  **Screen/files:** Seated guests, carried dish, ready VIP, selected guest;
  `src/ui/actors.rs::{draw_order_courses, draw_customer_sprite}`,
  `src/ui/dining.rs::draw_table`, `src/ui/guest_status.rs`, `src/commands.rs`.
  **Observed:** Saved gameplay shows long horizontal order chips reaching the
  floor edge and Serve controls behind portraits. Code draws table controls
  before guest sprites. The table inserts a small Serve hitbox, then the actor
  uses `entry(...).or_insert(sprite_rect)`, so the advertised sprite alternative
  does not replace/extend an existing table hitbox. Every seated guest also
  carries multiple meters, pips, name/type, course chips, and sometimes a bill;
  ready text and VIP occupy overlapping vertical bands by code geometry.
  **Change:** Compose one guest/order cluster after world actors. Show the
  immediate order, urgent patience/eating status, and a clear eligible Serve or
  VIP action; defer completed/full order history, bill, detailed satisfaction,
  and visit progression to the tap inspector. Keep meaningful non-color labels.
  Wrap/reflow orders within reserved bounds, keep warnings visible, and draw
  the selected inspector above other guests without covering its dismissal or
  service action. Use one intentional hit region or explicitly support both
  the visible action and portrait; do not accidentally discard either.
  **Acceptance:** Players can tell who needs which dish and when to serve;
  tapping what looks actionable produces the intended action. Guest details
  remain available without turning every guest into a dashboard.
  **Verify:** N and M with three-course orders, long names, all tables occupied,
  low patience, eating/waiting, ready VIP, and a pinned edge guest. Tap inspect,
  dismiss, Serve, and VIP without keyboard or hover. Depends on UI-01/UI-02.

- [ ] **UI-04 — Make the complete management catalog reachable with costs before commitment.**
  **Screen/files:** Management, clientele, crafting, upgrades, prestige choices;
  `src/ui/growth.rs`, `src/ui/clientele_board.rs`, `src/ui/prestige_modal.rs`,
  `src/commands.rs`, `assets/data/{upgrades,recipes,prestige_perks,customer_types}.json`.
  **Observed (code):** Growth displays only 2 of 9 upgrades, 4 of 13 recipes,
  and four ingredient lines; no alternate list is exposed. Prestige limits six
  perks to four. Clientele takes only rows fitting the height without scrolling
  or paging. Recipe tiles show shortened names and colored circles but omit
  ingredient costs, rewards, and unlock requirements; tapping dispatches Craft
  immediately, including locked tiles. The sidebar truncates clientele costs.
  **Change:** Build the management destination from UI-01 with deliberate
  touch-scroll/paging and persistent back/close access. Make all catalog entries
  reachable, showing discovered/relevant entries first and deferring advanced
  detail contextually. Recipe selection opens readable details with full name,
  requirements, owned/required ingredients, outcome, and an explicit Craft
  action. Show upgrade effect/cost/cap, full Attract costs, and shortages beside
  their actions; preserve meaningful disabled reasons. Expose all prestige
  choices in its comparison flow. Remove silent `take` limits as navigation.
  **Acceptance:** Content is not inaccessible because of array position or
  viewport height; players understand what they spend and receive before acting.
  No expansion of the permanent service dashboard is required.
  **Verify:** At N and M reach the final entry in every collection using touch;
  test locked, short-of-resources, affordable, maxed, long-name, and large-value
  states. Craft/buy/attract updates the displayed balances exactly once.
  Depends on UI-01/UI-02; coordinate modal ownership with UI-05.

- [ ] **UI-05 — Give one overlay exclusive input and a safe reading/return flow.**
  **Screen/files:** Clientele, specialization, prestige, closing ledger;
  `src/app.rs::tick_playing`, `src/ui/layout.rs::draw_and_collect_hitboxes`,
  `src/ui/types.rs::UiActions`, `src/commands.rs::read_input_action`,
  `src/ui/{clientele_board,specialization,prestige_modal,day_summary}.rs`.
  **Observed (code):** Only ledger/prestige set the `paused` local; specialization
  and clientele still advance service and process keyboard shortcuts. Rendering
  accumulates hitboxes from underlying screens, and `read_input_action` checks
  prestige/specialization/board controls before its generic modal guard. A
  covered board toggle or underlying Attract hitbox can therefore remain active.
  Specialization below 900px uses two rows of cards sized from a single-row
  height budget (at 844x390, total cards height is 608px before headings).
  **Change:** Establish a single explicit active overlay with its own action
  set and matching draw/input priority. Queue competing decisions; suppress
  covered pointer and keyboard actions. Pause service while mandatory choices
  and management comparisons block the floor. Reflow or scroll the whole
  decision within the viewport, reserving headings and fixed navigation space.
  Preserve the ledger's clear Open Day action and the Lounge's focused payoff.
  **Acceptance:** Reading a mandatory choice cannot silently cost guests, only
  the visible overlay responds, and all choices/navigation fit at supported
  sizes. Closing returns to the same service state.
  **Verify:** N and M; open the board near day end and during a pending house
  choice, tap covered control locations, wait while reading, and exercise all
  visible return paths. Check keyboard parity separately. Include short-canvas
  ledger goals and all prestige options. Depends on UI-02/UI-04.

- [ ] **UI-06 — Make prestige a reviewable, cancelable decision and state its real consequences.**
  **Screen/files:** Prestige entry/comparison; `src/ui/prestige_modal.rs`,
  `src/ui/growth.rs::draw_prestige_card`, `src/ui/layout.rs::draw_header_tiles`,
  `src/gameplay.rs::{try_prestige, confirm_prestige}`,
  `src/state/progression.rs::prestige`, `src/commands.rs`, `assets/data/ui_text.json`.
  **Observed (code):** Entering the perk modal has no cancel action, and tapping
  any perk card commits the reset immediately. Copy describes what a perk
  preserves but does not summarize all state lost/retained. The header prestige
  progress uses the base requirement while the actionable card uses the growing
  `prestige_requirement`, creating disagreement after earlier prestiges.
  **Change:** Use one requirement source and one primary progress home. Separate
  selecting/reading a perk from a labelled final prestige action; provide Cancel
  back to service. Derive a concise reset/retention summary from actual reset
  behavior, including reward and selected perk, without promising a broader
  reset than the implementation performs.
  **Acceptance:** Players can inspect all perks, back out without mutation, and
  understand the consequences before committing; every progress indicator agrees.
  **Verify:** N and M at below/exact threshold and after a prior prestige; tap
  entry -> perk -> Cancel, then entry -> perk -> confirm. Verify a single reset
  and persistence of the promised state. Depends on UI-04/UI-05.

- [ ] **UI-07 — Add a quiet utility route reachable during service.**
  **Screen/files:** Playing -> pause/utilities -> Settings/Title -> resume;
  `src/app.rs::{AppScreen, tick_playing, tick_settings}`, `src/ui/types.rs`,
  `src/ui/menu.rs`, `src/commands.rs`.
  **Observed (code):** Playing has no Menu, Pause, Settings, or return-to-title
  command. Settings Back always targets Title. Thus the player cannot access
  sound/fullscreen settings through visible in-game controls after starting.
  This is missing navigation, not evidence that gameplay and navigation are
  currently grouped together. The saved title also gives New/Load and
  Settings/Exit equal visual treatment.
  **Change:** Add a quiet, labelled Menu/Pause control spatially separate from
  Cook/Serve, growth purchases, and prestige. Supply Resume, Help, Settings, and
  return-to-title paths, preserving the running session and reporting save
  failures before leaving. Restore the originating screen from Settings.
  On Title, emphasize entry to play and give Settings/Exit a quieter utility
  grouping; retain load-error feedback and recovery access.
  **Acceptance:** Touch players can pause, change sound/fullscreen, get help,
  and return to their current shift; utilities never resemble gameplay choices.
  **Verify:** N and M, carrying a dish and during an urgent order; tap pause ->
  Settings -> Back -> Resume and title/load recovery without keyboard. Test
  unavailable browser fullscreen/exit behavior without hiding a return path.
  Depends on UI-01/UI-05; provides the Help entry for UI-08.

- [ ] **UI-08 — Teach beside the action, retire completed help, and make explanations reopenable.**
  **Screen/files:** Onboarding and in-service help; `src/ui/tutorial_panel.rs`,
  `src/state/tutorial.rs`, `src/ui/layout.rs::draw_header_tiles`,
  `src/ui/common.rs::draw_tooltip`, `src/ui/dining.rs`, `src/commands.rs`,
  `assets/data/{tutorial,ui_text}.json`, `README.md`.
  **Observed:** Saved rush image shows the tutorial covering the active-event
  banner. Code fixes the tutorial at the floor top and offers Next/Skip but no
  reopen route. Completed steps already disappear—preserve that. The serving
  plaque permanently repeats loop instructions when nothing is selected;
  header explanations depend on mouse hover. Current nearby service text says
  only “Space / E: Serve”, and carry onboarding does not name the Carry control.
  **Change:** Anchor the current short prompt to the relevant control, reserve
  space so urgent event/order information remains readable, and remove idle
  loop prose after onboarding. Add visible Help and tap-pinned explanations
  with dismissal. Name Cook, Carry, Serve, VIP, Clear Carried Dish, and relevant
  touch gestures exactly; keyboard hints must include the visible alternative.
  Reopening help must not reset run progress or require a new game.
  **Acceptance:** A new touch player completes the loop from visible prompts;
  an experienced player sees no permanent tutorial wallpaper and can retrieve
  an explanation when needed.
  **Verify:** N and M, all eight steps, Skip, completion, reload, reopen/dismiss,
  and rush/low-patience while a prompt is present. Depends on UI-01/UI-03/UI-07.

- [ ] **UI-09 — Separate transient events from persistent current state.**
  **Screen/files:** Service HUD and recent history; `src/ui/growth.rs::draw_event_feed`,
  `src/state.rs::add_message`, `src/ui/floaters.rs`, `src/ui/dining.rs`,
  `src/ui/dining/room.rs::draw_last_meal_lounge`, `src/ui/layout.rs`.
  **Observed:** Messages are untimed strings (last 18 retained), while the footer
  always displays up to five truncated pills with no full-history disclosure.
  Saved gameplay includes old arrivals and “New game started” as permanent
  attention surfaces. Header Lounge says “ready” whenever not busy, while the
  floor can say “locked”; these are different availability meanings. Current
  active-event banner sizes itself to unbounded text width.
  **Change:** Replace the permanent event strip with short-lived, prioritized
  feedback and a quiet, tap-open readable recent-history view. Keep current dish
  readiness, urgent trait warnings, event name/time/effect, and relevant current
  totals visible near their decisions; do not expire active hazards as toasts.
  Preserve critical errors until acknowledged or retrievable. Consolidate Lounge
  state into truthful locked/available/eligible/busy wording near its action and
  eliminate duplicate status homes. Bound/wrap active-event text without hiding
  its gameplay consequence. Retain local reward floaters where they help.
  **Acceptance:** Old arrivals stop competing with current orders; events remain
  understandable after animation through updated state or readable history.
  Players do not confuse an idle Lounge with an actionable VIP invitation.
  **Verify:** N and M during a quiet wait, rapid serves, rush, failed craft/save,
  and Lounge cooldown/eligibility transitions; tap history, read full messages,
  dismiss, and resume. Depends on UI-01/UI-03/UI-07.

- [ ] **UI-10 — Describe house-style effects by actual direction, independently of benefit color.**
  **Screen/files:** Specialization comparison;
  `src/ui/specialization.rs::{draw_specialization_card, effect_reads_as_buff, describe_effect}`,
  `assets/data/{specializations,ui_text}.json`.
  **Observed (code and saved specialization image):** The sign is chosen from
  whether the effect is beneficial, while the displayed amount uses its absolute
  value. Sweet Parlor's +0.10 cooking-time multiplier is displayed as “- 10%
  cooking time”; Rustic Hearth's -0.12 is displayed as “+ 12% cooking time”.
  This reverses the mechanical meaning of a consequential choice.
  **Change:** Use explicit directional wording such as “Cooking takes 10%
  longer” or “Cooking takes 12% less time”, and independently style benefit/
  drawback. Apply the same rule to spawn interval, patience, appetite decay,
  and yield. Show all trade-offs before selecting; keep choices equally prominent.
  **Acceptance:** A player understands each benefit and cost without decoding
  color or a misleading sign; wording agrees with the actual multiplier.
  **Verify:** Compare every displayed effect with its data and engine use, then
  inspect the three styles at N and M with touch selection. Add a focused
  direction/wording regression check if extracting a pure formatter.
  Can proceed independently of composition; final layout depends on UI-05.

### Further inspection — not established as live visual/input defects

- [ ] **UI-V1 — Refresh the evidence and complete the current-build interaction matrix.**
  **Screen/files:** All supported scenes; `scripts/capture_ui.ps1`,
  `src/app.rs::{begin_capture_scene, seed_gameplay_demo}`, `docs/verification/`,
  `README.md`, and this checklist.
  **Gap:** Existing captures use one canvas size and visibly lag current code.
  The seeded “dining_rush” image still has three tables and an active tutorial;
  it is not evidence for maximum-capacity late play. No reviewed screenshot
  establishes prestige, expanded guest inspector, portrait, or touch behavior.
  **Action:** Capture the current build at N/1280x720/M, including title/settings,
  fresh tutorial, tutorial-complete service, carrying, selected edge guest,
  maximum capacity, low patience/trait warning, clientele scrolling, crafting,
  specialization, all prestige choices, ledger, and Lounge. Extend deterministic
  capture fixtures only for real supported states; do not edit production state
  rules to manufacture a screenshot. Inspect existing white-backed portraits
  and high-contrast floor seams in a fresh build before deciding whether asset
  or rendering repair is needed. Check narrow modal/ledger text and actual
  browser input mapping rather than treating code arithmetic as visual proof.
  **Acceptance:** Each implementation task has current evidence and an honest
  pass/fail/blocked interaction result; new defects become concrete follow-ups,
  and older visual observations are retired when no longer reproducible.
  **Verify:** Run touch-only New/Load -> Cook -> Carry -> inspect -> Serve ->
  Clear -> VIP -> return, plus management, pause/settings/help, modal cancellation,
  prestige, and next day. Repeat after resize and browser display scaling.
  Report unsupported states/devices and publish result. Begin baseline captures
  before implementation, then replace affected captures as UI-01–UI-10 land.
