# Feast Frenzy

<!--
Editorial stance (decided 2026-07-08): this README keeps the "wholesome café"
framing as deliberate marketing misdirection. The fattening / Last Meal Lounge
premise is meant to be discovered in play, not spoiled here. Keep new copy cozy
on the surface; the dark hook lives in the game itself.
-->

Feast Frenzy is a cozy restaurant management game about running a little café
where every guest leaves fuller than they arrived. Cook house recipes, learn
your regulars by name, and grow a humble three-table dining room into a
renowned establishment — one satisfied guest at a time.

## Features

- **A full day of service** — a real-time day cycle with a closing ledger
  every night: takings, guests served, best streaks, and tomorrow's goal.
- **Guests with personality** — a persistent, named clientele across 14 types
  and 5 tiers; regulars come back, chat at their tables, and are always
  welcomed warmly. Every guest type has its own quirk to play around: foxes
  eye the pass, monkeys get cranky, big tippers reward good service.
- **A kitchen with a clock** — four stations, dish freshness on the pass,
  order courses, combo streaks, and full-house bonuses reward tight service.
- **The house specialty** — loyal guests are eventually invited to the
  exclusive Last Meal Lounge for a private final course, stocking the larder
  with the premium ingredients that attract an even finer clientele.
- **Make the house yours** — three café specializations with real trade-offs,
  nine upgrade tracks, twelve unlockable recipes, twenty achievements, and a
  prestige system with permanent perks.
- **Dining events** — dinner rushes, incognito critics, generous evenings,
  and the occasional health inspector keep every day a little different.
- **A guided first shift** — an eight-step tutorial teaches the loop, and
  every meter in the house explains itself on hover or with a pinned guest
  detail tap.

## Controls

The game is touch-first: tap the visible Cook, Serve, VIP, Clear Carried Dish,
and Prestige buttons. Tap the `?` badge beside a guest to pin their
satisfaction, patience, meal rhythm, and Lounge progress; tap it again to
dismiss the details.

- Left Click / tap: interact with restaurant UI.
- WASD / arrows: walk the chef around the floor.
- Space / E: interact with the nearest station or guest.
- 1–4: start each kitchen station.
- C: clear the carried dish.
- P: prestige when the renown bar is full.

## Interface support contract

The service floor is composed for landscape play. The supported review targets
are 1920x1080 normal, 1280x720 desktop, and 844x390 minimum landscape; the
embedded browser canvas should preserve those ratios while scaling. Portrait
windows show a readable rotate-to-landscape notice with the Menu recovery path.

The service screen gives the dining floor the dominant attention, keeps the
kitchen and carried dish immediately available, and moves clientele, upgrades,
recipes, and prestige into the on-demand Manage destination. Closing ledgers,
house style, prestige, Lounge rewards, help, and history are modal decisions
with their own return actions.

## Screen briefs

- **Service:** identify the next Cook, Carry, Serve, or VIP decision from the
  floor and station strip; current hazards stay near the affected guest or
  event, while old messages remain available through History.
- **Management:** compare the complete Clientele, Upgrades, Recipes, and
  Prestige catalogs before committing; paging and Back to Service remain
  visible at the decision surface.
- **Modal decisions:** one overlay owns input at a time. Mandatory choices
  pause service, explain their consequence, and expose an explicit cancel,
  close, or next-step action.

## Goal

Keep guests fed, chain efficient service, climb the clientele ladder, and
prestige into stronger future runs — the house remembers what you keep.
