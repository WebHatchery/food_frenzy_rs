# TODO — Feast Frenzy

## UI implementation record — 2026-09-20

The actionable UI audit tasks from the previous review are complete and have
been removed from the active queue. The implementation now has:

- a floor-first service composition with an on-demand Manage destination;
- explicit landscape breakpoints, compact station controls, touch-sized guest
  actions, a portrait rotate notice, and no negative compact cooking bars;
- one readable next-course cluster per guest, a pinned full-order inspector,
  unobscured Serve/VIP actions, and a 44px-class guest detail target;
- paged Clientele, Upgrades, Recipes, and Prestige catalogs, recipe detail
  review before crafting, visible costs/requirements, and all catalog entries
  reachable without array-position truncation;
- exclusive input ownership for Management, Help, History, specialization,
  prestige, and the closing ledger, with safe return paths;
- reviewable and cancelable prestige with an explicit reset summary, one
  requirement source, and a final confirmation action;
- a quiet in-service Menu route to Resume, Help, Settings, and save-and-return
  to Title, with Title entry actions emphasized over utilities;
- action-adjacent tutorial wording, reopenable Help, and completed tutorial
  steps that retire without leaving permanent loop instructions;
- quiet recent-service history, bounded event banners, truthful Lounge state,
  and directionally accurate house-style effects with regression coverage.

## Evidence

Fresh captures are stored directly in `docs/verification/`. Normal captures
use the host-reported 1920x1061 framebuffer even when the capture request is
1920x1080. Desktop captures are 1280x720, minimum landscape captures are
844x390, and the portrait capture is 390x844 with the rotate-to-landscape
notice. The minimum set covers service, rush, management, upgrades, recipe
detail, specialization, prestige, ledger, pause, Help, and guest inspection;
normal captures also cover title, settings, history, Lounge, and the same
decision screens at full size.

## Remaining verification gap

- [ ] **UI-V1 — Complete the live browser interaction matrix.** The current
  code and capture evidence cover layout, paging, state ownership, and visible
  touch targets, but this environment exposed no controllable native or
  embedded browser surface for a real tap-only Cook → Carry → inspect → Serve
  → Clear → VIP → pause/settings/help/return sequence, browser display-scale
  checks, or high-DPI picking. Re-run those interactions in the published
  browser build when a controllable browser surface is available; preserve the
  current screenshots and add only concrete newly reproduced defects.
