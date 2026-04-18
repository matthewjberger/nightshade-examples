# Lighthouse

A data-driven text adventure built on Nightshade's TUI backend.

Three-layer architecture:

```
 src/data/         -- pure authored types (rooms, items, rules, conditions, effects, ...)
 src/engine/       -- interpreter over (World, RuntimeState); no game knowledge
 src/game/
    areas/         -- per-area content; each file owns that area's rooms, items,
                      rules, NPCs, dialogues, and timers
    conditions.rs  -- shared predicate table (Condition::Ref targets)
    endings.rs     -- endings (span every area, so centralized)
    ids.rs         -- every entity ID for the adventure, registered as fns
    merge.rs       -- aggregator helper; panics on duplicate IDs across areas
    quests.rs      -- quest graph (spans every area, so centralized)
    texts.rs       -- shared text table
 src/view.rs       -- Nightshade State impl; scrolling transcript with parser prompt
 src/view/input.rs -- typed-command parser
 src/main.rs       -- wiring
```

The engine is independent of any particular game. Everything below the
`src/game/` line is swappable for a different adventure.

## Running

Native terminal (default):

```
cargo run -p lighthouse
```

Windowed TUI:

```
cargo run -p lighthouse --no-default-features --features tui
```

Browser (wasm):

```
trunk serve --release --open --config examples-terminal/lighthouse/Trunk.toml
```

## Playing

Free-form commands: `take key`, `go north` (or just `n`), `look at the drip`,
`use tinderbox`, `read ledger`, `talk to stranger`. Matching is case-insensitive,
honours item synonyms, and accepts common verb aliases (`take`/`get`/`grab`,
`examine`/`x`/`inspect`, `look`/`l`, etc.).

Meta commands:

- `help` / `h` / `?` — list the actions currently available
- `undo` / `u` — roll back the last turn (up to 20 snapshots)
- `quit` / `q` / `ESC` — exit

## Authoring

The whole content graph for one place or subsystem lives in one file under
`src/game/areas/`. Existing areas: `shore`, `cottage`, `tower`, `cellar`,
`stranger`, `storm`, `setup`.

### Adding a room to an existing area

1. Add a `room_x()` constructor in `src/game/ids.rs`.
2. Open the area file (e.g. `src/game/areas/tower.rs`).
3. Append a `rooms.insert(ids::room_x(), Room::new(...).with_exit(...))`
   block. If a neighbouring room is in a different area, add the return
   exit in that area's file.
4. Rebuild. The load-time validator catches dangling exit targets before
   the game starts.

### Adding an item

1. Add an `item_x()` constructor in `src/game/ids.rs`.
2. Open the area where the item *primarily lives*. Append an
   `items.insert(ids::item_x(), Item::new(...).takeable().initially_in(...))`
   block. Use `.initially_carried()` or `.initially_held_by(npc_x())` if
   appropriate, or omit the placement entirely for items revealed by a
   rule (like the keeper's body).

### Adding a rule

Rules go in the area file matching the trigger's subject. Append an
`rules.insert(ids::rule_x(), Rule::on(trigger, effects))` block. For
triggers like `OnUse { item_tower_key, in_room: tower_base }`, that's
`areas/tower.rs`. Top-level `build_world` merges every area's rules and
panics on duplicate IDs.

### Adding a whole new area

1. Create `src/game/areas/<name>.rs` with `pub fn build() -> AreaContents`.
2. Declare it in `src/game/areas.rs`: add `pub mod <name>;` and append
   `<name>::build` to `all()`.

### Tracing rule fires

During authoring, turn on rule tracing to see every rule as it fires in the
transcript:

```rust
let mut engine = Engine::new(world)?;
engine.set_rule_tracing(true);
```

Each fire emits a `TranscriptEntry::System` line like
`[trace] rule 'cottage_unlocked' fired`.

## Architecture notes

- `World` is immutable once built; `RuntimeState` is the only thing that
  changes during a run and the only thing that serializes to a save file.
- All content lives in `BTreeMap<Id, Entity>` maps keyed by string-newtype
  IDs defined centrally in `src/game/ids.rs`. Entities reference each other
  by ID; no pointers or indices leak out of the data layer.
- `Condition`, `Effect`, and `Trigger` are closed `#[non_exhaustive]` enums
  interpreted by `engine::eval`, `engine::exec`, and `engine::dispatch`.
  Adding a new verb is a single-variant change in the data layer plus a
  handler in the engine — exhaustive matching guarantees nothing is missed.
- Rules are dispatched through a derived `RuleIndex` built at `Engine::new`;
  the index is never serialized.
- Quest transitions auto-evaluate each turn — active stages advance whenever
  any outgoing transition's condition holds, cascading up to 32 deep in a
  single turn.
- `Engine::new` runs load-time validation (dangling references, unreachable
  rooms, active quest stages without transitions, dialogue nodes without
  targets, exit conditions/texts). Content bugs surface at boot.
- Text is composable: `Text::Ref` pulls from a shared table,
  `Text::Conditional` branches on a condition, `Text::OneOf` picks a random
  variant, `Text::Flag`/`Text::Stat` interpolate current values.
- Saves are versioned. `engine::save::save` prepends a magic + `u16`
  version; `load` returns a typed `SaveError` on mismatch.

## Tests

28 total, split across two suites:

- `tests/playthrough.rs` — scripted happy-path, alt-ending, save round-trip,
  dialogue branch, and storm-timer expiration playthroughs.
- `tests/feature_coverage.rs` — isolated tests for every engine feature
  (`Condition::{Chance,DispositionAtLeast,ItemIsSomewhere,QuestReached,
  RuleFired}`, `Effect::{ClearTranscript,FireRule,ScheduleEvent,OfferChoices}`,
  `Text::{Flag,Stat,Ref}`, timer cancel-on, validator error paths, save
  round-trip).
- In-module unit tests in `src/view/input.rs` cover the typed-command
  parser.

```
cargo test -p lighthouse
```
