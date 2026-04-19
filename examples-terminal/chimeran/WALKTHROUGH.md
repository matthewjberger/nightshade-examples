# Chimeran — command walkthrough

Exact keystrokes to reach each of the five endings. Commands are shown
one-per-line; dialogue picks are shown as `1`/`2`/`3` (the number next
to the option on screen). Hit Enter after each command.

---

## Quick reference

### Map of exits

```
              [bedroom] --s--> [hallway] --e--> [kitchen]
                                 |
                                 w
                                 v
           [corridor] <--e-- [hallway]
                |
                d
                v
           [elevator] --d--> [lobby] --s--> [street] --e--> [office floor] --e--> [desk]
                ^               ^              |                                    |
                +-- u ----------+              +-- n to lobby                       |
                                                                                    w (teleports home)
                                                                                    v
                                                                              [bedroom]
```

### Commands the parser understands

- **Direction:** `n` `s` `e` `w` `u` `d` `ne` `nw` `se` `sw`, plus full words (`north`, `south`, ...).
- **Verbs:** `look` (`l`), `examine` (`x`), `take`, `drop`, `use`, `open`, `read`, `talk to`, `ask`.
- **Meta:** `help` / `h`, `undo` / `u`, `quit` / `q`, `inventory` / `i`, `wait` / `z`.
- **Inside a dialogue:** type the option's number (`1`, `2`, ...) or a
  distinctive word from its label. Numbers are always reliable.

### The daily loop (commit to muscle memory)

Bedroom → desk: `s`, `w`, `d`, `d`, `s`, `e`, `e`.
Desk → bedroom: `w` (the game teleports you home).
Sleep: `open bed`, then `1`.

---

## Ending 1 — "Time Passes. The Work Continues." (stasis)

Decline the exploit for three extra cycles past the planned ending.
Minimum input: *sleep ten times*.

Cycle 1 → 8 — repeat this block eight times (once per cycle):

```
s
w
d
d
s
e
e
w
open bed
1
```

At cycle 8, do the same three extra times:

```
s
w
d
d
s
e
e
w
open bed
1
```

After the third extra sleep (`cycle_stasis_loops == 3` and `cycle == 11`),
the ending fires. You'll see **Time Passes. The Work Continues.**

---

## Ending 2 — "Substrate Coherence Collapse" (collapse)

Let awareness cross the threshold before running the exploit. The
fastest path: look closer in the mirror once you reach cycle 5, don't
run `check.py`, then keep poking.

Cycles 1 → 5 — eight sleeps as above. Then, at cycle 5, **before**
sleeping:

```
open mirror
1
```

`1` is "Look closer." → `+2 AWA` → mirror-looked-closer flag set.

If AWA is now ≥ 6 the collapse ending fires next turn end. If not, keep
sleeping through cycles 5–7 and poke at things (wait, look around the
apartment) to nudge awareness. The first turn end after AWA ≥ 6 and
cycle ≥ 5 triggers **Substrate Coherence Collapse**.

---

## Ending 3 — "Chimeran Will Continue." (neutral)

Run the exploit, reach the redux, sleep. Skip the messages.

### Cycles 1–7 — seven identical sleeps

```
s
w
d
d
s
e
e
w
open bed
1
```

Repeat seven times. You are now waking in cycle 8.

### Cycle 8 — the exploit

```
s
w
d
d
s
e
e
```

You're at the desk. The `internal-1847` email has arrived.

```
open mail
```

You're in the Mail dialogue. Read the exploit email (it's at or near
the top of the inbox — pick the numbered option whose label mentions
`internal-1847` or `please run this`):

```
1
```

(Exact number depends on which other inbox items are pending. Look at
the on-screen menu, pick the one labeled as from `internal-1847`.)

Read through any reply prompt, then close:

```
close
```

Now the Code tool:

```
open code
1
```

`1` is **Run check.py**. The exploit runs. `stat_exploit_counter` is
now 25 and the reveal window is open.

Close Code and hit the four reveals:

```
close
open research
```

Pick **Query the substrate** (whichever number it is — look at the
menu):

```
1
```

Then back out:

```
close
open reference
```

Pick **Source index**:

```
1
```

Back out, open Notepad (opening it with Unstripped enabled
auto-marks that flag — you just need to be inside the dialogue):

```
close
open notepad
```

Pick any of the notes (e.g. `groceries`):

```
1
close
```

Finally the picture frame:

```
open picture frame
```

Pick **Who is this** (the option that unlocked after the exploit):

```
2
```

(Exact number depends on visible options; pick the line reading "Who
is this.") Then back out:

```
1
```

(or whatever closes the memory view).

The reveal window closes on the next turn end. The case file prints.
You wake as Cameron 0048 in the bedroom — **redux begins**.

### Redux → neutral

You need to reach the bedroom's bed and conclude. You already are in
the bedroom, so:

```
open bed
```

Pick **Conclude the redux. Let the next instance begin.** (it's the
one referencing "next instance"):

```
1
```

**Chimeran Will Continue.** (neutral variant) prints.

---

## Ending 4 — "Chimeran Will Continue." (good)

Prerequisite: Marisol-rel must be ≥ 2 before the exploit runs,
otherwise Notepad's "Leave something for the next instance" option
stays locked. Earn it via Chatter during the normal cycles.

### Setup during cycles 1–7

On any cycle while at the desk, open Chatter, find Marisol's DM, pick
the warm reply option (the one with parenthetical positive flavour —
e.g. "(reply warmly)"):

```
open chatter
```

Navigate to the Marisol channel (usually option `1` or `2`). Pick the
warm/engaged reply option to bump her relationship. Do this at least
twice across cycles 1–6 to push `marisol_rel` to 2.

### Then cycles 1–7 sleeps as in the neutral path

### Cycle 8 — exploit as above, then before closing the reveal window

Instead of opening Notepad's `groceries` note, pick the unlocked
message option:

```
open notepad
```

Pick **(+) Leave something for the next instance** (appears because
`marisol_rel ≥ 2` AND reveal window open AND not yet sent):

```
[number next to "Leave something for the next instance"]
```

Then pick one of the four message variants (`1`, `2`, `3`, or `4` — each
interpolates different ending text):

```
1
close
```

Then hit the remaining reveals as in the neutral path (research,
reference, picture frame). The window closes, redux begins.

### Redux → conclude

```
open bed
1
```

**Chimeran Will Continue.** (good variant) prints.

---

## Ending 5 — "So Will You." (best)

Same as good, plus the Rachel message in redux.

### Setup: the good-ending setup, plus read every Rachel email

Each cycle, open Mail and read the Rachel-from-Rachel email (it's in
the inbox, one new one each cycle). Reply when prompted. This builds
`rachel_rel` the same way Marisol's does.

### Cycle 8 exploit as in the good path

Send the next-instance message via Notepad during the reveal window
(same as good ending).

### Redux — send Rachel too

After `check.py` has closed the window and you're back in the bedroom:

```
s
w
d
d
s
e
e
open chatter
```

Rachel's DM is now reachable. Pick the Rachel channel (look for her
name in the menu). Pick the message-to-Rachel option (this one only
appears when `flag_next_instance_message_sent` is already set — which
it is if you did the Notepad step):

```
[number next to Rachel's DM]
[number of the message variant you want to send]
close
w
open bed
1
```

**So Will You.** prints. Best ending.

---

## Notes for testers

- **The parser is forgiving.** `open the mirror`, `open mirror`,
  `examine mirror`, and `x mirror` all work. Compound phrases (`pick
  up the key`, `look at the sign`, `turn on the lamp`) route correctly.
- **If something doesn't match, try a number.** Inside a dialogue, the
  options are numbered in the order they appear. `1` is always the
  top visible option.
- **`undo` (`u`) rewinds one command.** The undo stack holds 20 turns.
- **`help` lists every currently available command.**
- **Examine any noun the prose mentions.** The bedroom alone has ~20
  examinable surfaces (`nightstand`, `drawer`, `pillow`, `curtains`,
  `ceiling`, `floor`, `walls`, `air`, `light`, ...). This is flavour,
  not gameplay — but try `examine calendar` on cycle 1 for the starting
  date, and watch it skip days as the cycles advance.

## Troubleshooting

- **"You can't do that here"** — verb recognised, no target. Try a
  different noun, or `help` to see available actions.
- **"Which one? Be more specific."** — ambiguous. Use a more specific
  noun, or the numbered option if inside a dialogue.
- **"Nothing happens."** / **"That wouldn't taste of anything worth
  knowing."** / **"Violence won't help you here."** — verb-specific
  refusals. The game recognised the verb but the target is wrong.
- **Bed option "Get into bed. Sleep." is greyed out** — you haven't
  been to the desk this cycle. Travel east until `east (into your
  office)` shows up, enter it, and try again.
