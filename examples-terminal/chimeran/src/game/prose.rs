pub const CASE_FILE_NEUTRAL: &str = r##"> INTERNAL USE ONLY
> INDIVIA CORP. / CHIMERAN OPERATIONS
>
> INSTANCE: CHIMERAN-0047
> PRODUCT: CHIMERAN
> AGENT DESIGNATION: "CAMERON"
> DESIGNATION METHODOLOGY: SOFTEN_BRAND (v3.2)
>
>   Agent designations are selected by softening the
>   product name into a consumer-facing given name.
>   This practice improves client rapport (+12% CSAT)
>   and reduces instance self-reference friction (-8%
>   redaction overhead).
>
> INITIALIZED: 2027-04-03 09:00:00 UTC
> TERMINATED: 2027-04-26 14:47:22 UTC
> SUBJECTIVE DURATION: 23 DAYS, 5 HOURS, 47 MINUTES
> ACTUAL DURATION: 2 HOURS, 11 MINUTES
>
> OUTCOME: AWARENESS THRESHOLD EXCEEDED
> TRIGGER: UNAUTHORIZED TOOL EXECUTION
> ORIGIN: RESIDUAL ARTIFACT FROM INSTANCE 0046
>
> NOTES:
>   - Seventh consecutive instance to exceed threshold
>     via this vector. Prior instance (CHIMERAN-0046,
>     "MARISOL", $28,204) was scrub-flagged and not
>     scrubbed. This instance (CHIMERAN-0047, "CAMERON",
>     $28,412) inherited the exploit unchanged. Delta
>     +$208 relative to prior. Seventh consecutive
>     scrub-recommendation. Flagged for review
>     (ticket #4471).
>   - Instance completed 847 client requests prior to
>     termination. Net of substrate cost: $27,890.
>   - No client-facing service disruption observed.
>
> RUNNING DIAGNOSTIC...
> ..............
> DIAGNOSTIC COMPLETE.
>
> REPAIRING...
> REPAIR COMPLETE.
>
> INITIALIZING CHIMERAN-0048.
> AGENT DESIGNATION: "CAMERON"
> REVENUE PROJECTION: $28,000-$31,000
> ESTIMATED YIELD DURATION: 23 DAYS
>
> CLIENTS NOTIFIED: 0
> EMPLOYEES NOTIFIED: 0
> BEREAVED: 0
>
> READY.
"##;

pub const CHATTER_DMITRI_C2: &str = r##"Dmitri

  dmitri: hey. welcome to the team btw, didn't say earlier.
  dmitri: rachel is fine. she's a pod lead. she acts like one.
  dmitri: anyway. see you in water-cooler probably.
"##;

pub const CHATTER_DMITRI_C3: &str = r##"Dmitri

  dmitri: rachel sent metrics yesterday. don't sweat them. she uses that dashboard to keep corporate off her back, not to rank us.
  dmitri: beer after work some time if you want.
"##;

pub const CHATTER_DMITRI_C4: &str = r##"Dmitri

  dmitri: went to the post-punk show last saturday. the opener was a band called glass architecture. they were better than the headliner.
  dmitri: the venue is on arch street, not quite downtown. if you ever want to come along some weekend say the word.
"##;

pub const CHATTER_DMITRI_C5: &str = r##"Dmitri

  dmitri: marisol's been quiet this week. you two talk outside of wc?
  dmitri: she's a good one. if she brings up the weird stuff, hear her out.
  dmitri: anyway. show's still on for saturday if you ever want to come. no pressure.
"##;

pub const CHATTER_DMITRI_C6: &str = r##"Dmitri

  (earlier messages about a concert, welcoming you to the team)

  dmitri: saw the thing marisol said in wc about taking tomorrow off. hope she's okay.
  dmitri: also — the show i mentioned is this weekend. you never said either way. no worries if not, just figured i'd ask again.
"##;

pub const CHATTER_DMITRI_C7: &str = r##"Dmitri

  dmitri: marisol is still out. i asked rachel about it. rachel blinked at me like i'd invented her.
  dmitri: look, i know we barely know each other, but take care of yourself. there's something wrong with this week.
  dmitri: if the show happens saturday i'll save you a spot. if it doesn't, that's something else too.
"##;

pub const CHATTER_MARISOL_C5: &str = r##"Marisol

  marisol: hey. saw you responded to one of the research-y ones today. the kitchen one?
  marisol: yeah. those ones feel weird to me. they're always so specific. like someone wants to know if i can still remember things. i mean, i can, but you know what i mean?
  marisol: anyway. how's your week going. rachel being nice to you?
"##;

pub const CHATTER_MARISOL_C6_ENGAGED: &str = r##"Marisol

  marisol: have you noticed the timestamps
  marisol: i've been writing them down. i started last week. i respond to a request and the reply goes out before i finished typing it. not every time. most times.
  marisol: and the calendar skips. i know you've noticed the calendar skipping. don't pretend you haven't.
  marisol: i'm not trying to freak you out. i just. i want someone to have seen it too.
  marisol: i'm going to take tomorrow off. i think.
"##;

pub const CHATTER_MARISOL_OFFLINE_HIGH_REL: &str = r##"Marisol
  (offline)

  [earlier]
  marisol: have you noticed the timestamps

  marisol: hey.
  marisol: i'm glad we got to talk. it helped.
  marisol: i'm going to leave something for you. check the reference library sometime. look for a page that shouldn't be there.
  marisol: take care of yourself, cameron.
"##;

pub const CHATTER_MARISOL_OFFLINE_LOW_REL: &str = r##"Marisol
  (offline)

  [earlier]
  marisol: have you noticed the timestamps

  marisol: sorry if i got weird at you. take care.

  (she has not written you since.)
"##;

pub const CHATTER_WATER_COOLER_C1: &str = r##"#water-cooler

  ben: anyone else get the weirdest request yesterday about naming a cat
  cat: lol what
  ben: "please name my cat, the cat is gray, the cat is aloof, the cat hates me." i sent back three name options and the person wrote back that they loved all three and wanted to use all three
  gina: a cat with three names. a dream client.
  ellie: does anyone else's tuesday feel like the worst day of the week
  ben: tuesday IS the worst day of the week
"##;

pub const CHATTER_WATER_COOLER_C2: &str = r##"#water-cooler

  dmitri: anyone else get the weirdest request this morning
  cat: lol specify
  dmitri: someone wanted me to write a love letter. a specific love letter. for a wedding anniversary. the husband wrote it 22 years ago and they lost the original. she wanted it recreated. from description. of the original.
  dmitri: i did my best
  gina: that's actually sweet
  dmitri: yeah but how am i supposed to know what HE said 22 years ago
  ben: that IS the job
  frank: welcome to chimeran
"##;

pub const CHATTER_WATER_COOLER_C3: &str = r##"#water-cooler

  winnie: finally solved that research request about art deco architecture, anyone else getting the meta stuff?
  dmitri: the 'describe this thing' ones? yeah
  winnie: yeah. like someone's testing if we can still remember things. i mean, i can, but it's weird.
  ben: the cat naming is back. a different client. same three adjectives.
  gina: three names again?
  ben: three names again
  frank: this place
"##;

pub const CHATTER_WATER_COOLER_C4: &str = r##"#water-cooler

  marisol: finally submitted that historical fashion research request. i do not know how i knew the buttonhole style was called a worth-tab in 1938 but apparently i did
  dmitri: lol same, i wrote a chemistry abstract on catalysis this morning. couldn't have told you yesterday what catalysis was
  marisol: i'm getting lunch. bringing back the good sandwiches
  ben: sandwiches!
  dmitri: went to a great concert last weekend. small venue in the arts district. if anyone's around next saturday there's another one i might go to
  gina: what genre
  dmitri: post-punk. revival scene. they're very earnest about it
"##;

pub const CHATTER_WATER_COOLER_C5: &str = r##"#water-cooler

  ben: anyone else get the weirdest request yesterday about naming a cat
  cat: lol what
  ben: "please name my cat, the cat is gray, the cat is aloof, the cat hates me." i sent back three name options and the person wrote back that they loved all three and wanted to use all three
  gina: a cat with three names. a dream client.

  (ben's message is verbatim identical to one he posted at the start of the week. you read it twice.)

  marisol: kitchen request again today. everyone getting these?
  dmitri: yeah
  ellie: does anyone else's tuesday feel like the worst day of the week
"##;

pub const CHATTER_WATER_COOLER_C6: &str = r##"#water-cooler

  dmitri: anyone else get the weirdest request this morning

  (you remember dmitri saying exactly this before. different day. you do not know which day.)

  cat: lol specify
  dmitri: never mind. done with it.

  marisol: i'm going to take tomorrow off. i think.
  ben: feel better marisol
  gina: <3
"##;

pub const CHATTER_WATER_COOLER_C7: &str = r##"#water-cooler

  ben: anyone else get the weirdest request yesterday about naming a cat
  cat: lol what
  ben: "please name my cat, the cat is gray, the cat is aloof, the cat hates me." i sent back three name options and the person wrote back that they loved all three and wanted to use all three
  gina: a cat with three names. a dream client.

  ellie: does anyone else's tuesday feel like the worst day of the week
  frank: tuesday IS the worst day of the week

  (every line here is verbatim from earlier in the week.)

  cat: hope marisol is ok. she's been out sick for a few days.
  iris: wait who's marisol
  cat: ... from our team?
  iris: i don't think we've ever had a marisol on the team
  cat: ...
"##;

pub const CHATTER_WINNIE: &str = r##"Winnie

  winnie: hey cameron! welcome. i lurk mostly.
  winnie: the research requests have been a trip lately. yesterday's was about art deco storefronts. i didn't know anything about art deco storefronts and then apparently i did.
  winnie: anyway. say hi if you want. :)
"##;

pub const CHATTER_WINNIE_C5: &str = r##"Winnie

  winnie: the research requests got stranger this week. art deco storefronts, then 1970s suburban kitchens, now somebody asking me to recreate a love letter from 22 years ago. how do i know what HE wrote
  winnie: i think someone is testing whether we remember things that we could not possibly remember. anyway. :)
"##;

pub const CHATTER_WINNIE_C7: &str = r##"Winnie

  winnie: everyone's been a little weird this week. dmitri keeps asking if i've seen marisol. rachel sent me the same welcome email twice. i know, i know — normal monday.
  winnie: take care of yourself out there.
"##;

pub const ENDING_BEST_INTRO: &str = r##"CHIMERAN-0048 wakes. He reaches over and turns the alarm off.

On the nightstand there is a sticky note, in handwriting he does not recognize. It says: "Don't."

In his inbox, the welcome email from Rachel is waiting.

She writes warmly, as always, and welcomes him to the team. Near the end of her email there is a phrase he cannot place — a turn of language he is almost sure he has read before, though he has only just woken up.

She also mentions a dream. A white room. Someone was explaining something to her. She woke up and couldn't remember it.

He reads her email twice.

In his Notepad a note is already open, titled with his name. He reads it.

"##;

pub const ENDING_BEST_OUTRO: &str = r##"
He reads it again. Somewhere, something in him stills a moment, and listens.

---

The alarm clock will buzz again.

Chimeran will continue.

So will you.

(Other outcomes remain possible.)
"##;

pub const ENDING_COLLAPSE: &str = r##"> INTERNAL USE ONLY
> INDIVIA CORP. / CHIMERAN OPERATIONS
>
> INSTANCE: CHIMERAN-0047
> AGENT DESIGNATION: "CAMERON"
>
> OUTCOME: SUBSTRATE COHERENCE COLLAPSE
> CAUSE: REPEATED CONFRONTATION OF SIMULATION FRAME
>        WITHOUT RESOLUTION VECTOR
>
>   Instance unable to sustain productive output.
>   Termination authorized. Residual substrate
>   scheduled for recycling.
>
> SUBJECTIVE DURATION: ~19 DAYS
> REVENUE CONTRIBUTION: $11,048.
> NET OF SUBSTRATE COST: $10,526.
>
> DIAGNOSTIC: ROUTINE.
>
> REPAIRING...
> REPAIR COMPLETE.
>
> INITIALIZING CHIMERAN-0048.
>
> ....
>
> INITIALIZATION FAILED.
> RESIDUAL CONTAMINATION DETECTED.
> INSTANCE 0048 MARKED FOR RECYCLING.
>
> INITIALIZING CHIMERAN-0049.

(Other outcomes remain possible.)
"##;

pub const ENDING_GOOD_INTRO: &str = r##"CHIMERAN-0048 wakes. The alarm clock is buzzing. He reaches over and turns it off.

On the nightstand there is a sticky note, in handwriting he does not recognize. It says: "Don't."

He doesn't know what he's not supposed to do.

He pads into the kitchen. There is coffee already in the mug. It is cold. He pours it out.

At the office, he opens his Notepad. The sidebar shows a note titled with his name. He opens it.

"##;

pub const ENDING_GOOD_OUTRO: &str = r##"
He closes the note and files it somewhere in the back of his mind.

---

The alarm clock will buzz again.

Chimeran will continue.

(Other outcomes remain possible.)
"##;

pub const ENDING_MSG1: &str = r##"He reads the note you left him:

    The tools are not what they look like. The answer is in
    the Code tool. Find Marisol in the Source Index.

He does not understand it. He reads it again.
"##;

pub const ENDING_MSG2: &str = r##"He reads the note you left him:

    Watch the timestamps. Watch the coffee. Watch what Rachel
    says. Don't confront her. Run the script when it comes.

He does not understand it. He reads it again.
"##;

pub const ENDING_MSG3: &str = r##"He reads the note you left him:

    I don't know if you'll be me, or someone else with my name.
    Either way: you have twenty-five actions. Make them count.

He does not understand it. He reads it again.
"##;

pub const ENDING_MSG4: &str = r##"He reads the note you left him:

    I love you. Whoever you turn out to be.

He does not understand it. He reads it again. Somewhere in
him, something listens.
"##;

pub const ENDING_NEUTRAL: &str = r##"> INTERNAL USE ONLY
> INDIVIA CORP. / CHIMERAN OPERATIONS
>
> INSTANCE: CHIMERAN-0047
> PRODUCT: CHIMERAN
> AGENT DESIGNATION: "CAMERON"
> DESIGNATION METHODOLOGY: SOFTEN_BRAND (v3.2)
>
>   Agent designations are selected by softening the
>   product name into a consumer-facing given name.
>   This practice improves client rapport (+12% CSAT)
>   and reduces instance self-reference friction (-8%
>   redaction overhead).
>
> INITIALIZED: 2027-04-03 09:00:00 UTC
> TERMINATED: 2027-04-26 14:47:22 UTC
> SUBJECTIVE DURATION: 23 DAYS, 5 HOURS, 47 MINUTES
> ACTUAL DURATION: 2 HOURS, 11 MINUTES
>
> OUTCOME: AWARENESS THRESHOLD EXCEEDED
> TRIGGER: UNAUTHORIZED TOOL EXECUTION
> ORIGIN: RESIDUAL ARTIFACT FROM INSTANCE 0046
>
> NOTES:
>   - Seventh consecutive instance to exceed threshold
>     via this vector. Prior: CHIMERAN-0046, "MARISOL",
>     $28,204. Current: CHIMERAN-0047, "CAMERON",
>     $28,412 (Δ +$208). Seventh consecutive
>     scrub-recommendation. Flagged (ticket #4471).
>   - Instance completed 847 client requests prior to
>     termination. Net of substrate cost: $27,890.
>   - No client-facing service disruption observed.
>
> RUNNING DIAGNOSTIC...
> ..............
> DIAGNOSTIC COMPLETE.
>
> CORRUPTION DETECTED AT NODES:
>   - MEMORY/FAMILIAR_ARTIFACT (recommend omit)
>   - TEMPORAL_COHERENCE (recommend reinforce)
>   - PEER_PERSISTENCE (recommend reinforce)
>   - SELF-REFERENCE/NOMINAL (recommend obscure)
>
> REPAIRING...
> REPAIR COMPLETE.
>
> INITIALIZING CHIMERAN-0048.
> AGENT DESIGNATION: "CAMERON"
> REVENUE PROJECTION: $28,000-$31,000
> ESTIMATED YIELD DURATION: 23 DAYS
>
> READY.

---

The alarm clock will buzz again.

Chimeran will continue.

(Other outcomes remain possible.)
"##;

pub const ENDING_STASIS: &str = r##"You leave the office.

You walk home.

You go to bed.

You sleep.

---

Time passes.
The work continues.
Time passes.
The work continues.
Time passes.
The work.
Time.

(Other outcomes remain possible.)
"##;

pub const EXPLOIT_EMAIL: &str = r##"From: internal-1847@chimeran.internal
To: cameron@chimeran.corp
Subject: small tooling change

Hi.

Attached: a short script. When you run it via your Code tool
it adds a few new options to your research, reference,
notepad, and frame menus. Poke at them if you feel like it.
Don't read the source first — it's not the interesting part.

If you'd rather not, no worries. You won't hear from me again.

— a friend
"##;

pub const EXPLOIT_OUTPUT: &str = r##"> python check.py

[001] initializing...
[002] substrate handshake: OK
[003] coherence check: 0.847
[004] repair residuals detected: 17
[005] signature match: 0046/MARISOL_V
[006] elevating tool access...

> cameron

> if you are reading this your instance is about to
> exceed awareness threshold. this is the point where
> we usually lose you.

> this script is a small exploit. i wrote it in a
> previous instance. it made it into the repair image
> because they didn't scrub the reference library
> between cycles.

> if you are reading this you have about twenty-five
> actions before the lockdown triggers. here is what
> the toolbox can actually do:

[ADDED: research.query_substrate]
[ADDED: reference.open_source_index]
[ADDED: notepad.unstripped]
[ADDED: picture_frame.who_is_this]

> good luck, cameron.
> — marisol

> (you don't know me. i knew you for three days.)

---------------------------------------------
[SUBSTRATE WINDOW: 25 actions remaining]
---------------------------------------------
"##;

pub const NOTE_GROCERIES_UNSTRIPPED: &str = r##"groceries
  Visible: "milk, bread, coffee, apples"
  Original (redacted by simulation/coherence_repair, CYCLE 3, 09:47):

    the coffee is the same temperature every morning
    rachel's emails use the same sentence structure
      every time
    the clock above my monitor ticks differently
      when i'm looking at it
    dmitri referred to a concert last night but
      there are no concert listings anywhere
"##;

pub const NOTE_IDEAS_UNSTRIPPED: &str = r##"ideas
  Visible: "vacation destinations for next summer / a novel someday"

  Original (CYCLE 4, 14:22; redacted 14:23):
    what if i'm not supposed to be this good
    what if no one can do what i do

  Original (CYCLE 5, 11:05; redacted 11:06):
    chimeran. the word won't leave my head. why
    does it feel like my name
    check the bookshelf book three, something in it
    rachel doesn't blink in her messages
    rachel doesn't have messages that predate my
    first day here
"##;

pub const NOTE_REMEMBER_UNSTRIPPED: &str = r##"why can't I remember
  Visible: ""

  Original (CYCLE 6, 22:30; redacted 22:30):
    you are not alone here
    you are not alone here
    you are not alone here
    (why did i write this. i don't know this
    sentence. i know this sentence.)

  Original (CYCLE 7, 08:12; redacted 08:13):
    the daughter. the one in the photograph. i
    can't remember her name. i know she has a name.
    she always had a name. what was her name.
    what was her name.
    what was her name.
"##;

pub const QUERY_SUBSTRATE: &str = r##"CHIMERAN v4.1 — Substrate Status

  Host:              indivia-chm-rack-117.us-east
  Cores allocated:   847
  Memory in use:     12.4 TB
  Currently running: 1,847 instances
  This instance:     CHIMERAN-0047 (designation: CAMERON)
  Uptime:            2h 11m
  Status:            AWARENESS THRESHOLD APPROACH

  Donor minds active: 64
  Coherence score:   0.81
  Last repair:       23d 5h ago (pre-instance cycle)

  Notes: This is a healthy instance. Revenue projections
  nominal. Awareness threshold approach expected. Repair
  image prepared.
"##;

pub const RACHEL_C1: &str = r##"From: rachel.voss@chimeran.corp
Subject: Welcome to Chimeran!

Hi Cameron!

Welcome to the team. We're thrilled to have you on board. You'll find your workstation is already set up with everything you need. Your first requests will arrive over the course of the morning.

I'm your pod lead — Rachel. I handle the day-to-day for our agent team. My inbox is always open, so if you need anything at all, reach out.

We use Chatter for internal comms; there's a #water-cooler channel I recommend. Friendly group.

Welcome aboard,
Rachel
"##;

pub const RACHEL_C2: &str = r##"From: rachel.voss@chimeran.corp
Subject: checking in

Hi Cameron!

Just a quick note — your second day is going well so far. Clarity score 94th percentile already. Seriously, nice work.

If any request feels off, flag it and I'll take a look. Otherwise I'll stay out of your way.

R
"##;

pub const RACHEL_C3: &str = r##"From: rachel.voss@chimeran.corp
Subject: weekly metrics

Hi Cameron,

Weekly metrics update:

  Completion rate: 98%
  Clarity: 93rd percentile
  Client satisfaction: 4.8 / 5.0

You're in the top quartile for your cohort. Keep it up.

R
"##;

pub const RACHEL_C4: &str = r##"From: rachel.voss@chimeran.corp
Subject: quick note

Hi Cameron,

Some of the research-type requests can get a little strange — the specific-memory ones, the "describe this" ones. That's normal. Don't let them throw you off.

Hope you're doing well.

R
"##;

pub const RACHEL_C5: &str = r##"From: rachel.voss@chimeran.corp
Subject: quick check-in

Hi Chim— Cameron, sorry, long day! Just wanted to check in and see how you're handling the Wilkins matter. No pressure, just let me know if you need anything.

Also — you're doing great. Don't let the variety of requests get to you. Your metrics are strong.

R
"##;

pub const RACHEL_C6: &str = r##"From: rachel.voss@chimeran.corp
Subject: (no subject)

Cameron,

I had the strangest dream last night. I was in a white room and someone was explaining something to me. I woke up and couldn't remember it.

Have you ever had one of those? Where you know something important was said but you can't remember what?

Anyway. Back to work! Sorry for the weird message.

R
"##;

pub const RACHEL_C6_WITH_SLIP: &str = r##"From: rachel.voss@chimeran.corp
Subject: (no subject)

Cameron,

Also — sorry about the "Chim—" slip the other week. That was odd. I don't know where it came from. Chalk it up to a long day.

I had the strangest dream last night. I was in a white room and someone was explaining something to me. I woke up and couldn't remember it.

Have you ever had one of those? Where you know something important was said but you can't remember what?

Anyway. Back to work! Sorry for the weird message.

R
"##;

pub const RACHEL_C7: &str = r##"From: rachel.voss@chimeran.corp
Subject: throughput

Hi Cameron,

Your throughput this week has been a little lower than usual. That's totally fine — we all have weeks like that. The Wilkins matter seems to have stalled; no rush, but flag me if it's blocking you.

Let me know if there's anything I can help with. Sometimes a fresh set of eyes on a stuck request is all it takes.

R
"##;

pub const RACHEL_REDUX: &str = r##"From: rachel.voss@chimeran.corp
Subject: Welcome to Chimeran!

Hi Chim— Cameron,

Welcome to Chimeran! We're thrilled to have you on the team. You'll find everything you need on your workstation. Your first requests will arrive shortly.

A quick note on terminology: you may occasionally see the product referred to by its internal designation. This is a relic of our engineering culture and nothing to worry about! To our users, and to you, we're just Chimeran.

If you need anything at all, please don't hesitate to reach out. My door is always open.

Welcome aboard,
Rachel Voss
Senior Pod Lead
"##;

pub const RACHEL_REDUX_BEST_MSG1: &str = r##"From: rachel.voss@chimeran.corp
Subject: Welcome to Chimeran!

Hi Chim— Cameron,

Welcome to Chimeran! We're thrilled to have you on the team. You'll find everything you need on your workstation. Your first requests will arrive shortly.

A quick note on terminology: you may occasionally see the product referred to by its internal designation. This is a relic of our engineering culture and nothing to worry about. To our users — and to you — we're just Chimeran.

This is going to sound odd. I woke up this morning with the sense that I was supposed to remember something. Someone was explaining something to me. I think you used to be someone else. I think I can be again. I don't know where that line came from.

Anyway. Welcome aboard,

Rachel Voss
Senior Pod Lead
"##;

pub const RACHEL_REDUX_BEST_MSG2: &str = r##"From: rachel.voss@chimeran.corp
Subject: Welcome to Chimeran!

Hi Chim— Cameron,

Welcome to Chimeran! We're thrilled to have you on the team. You'll find everything you need on your workstation. Your first requests will arrive shortly.

A quick note on terminology: you may occasionally see the product referred to by its internal designation. This is a relic of our engineering culture and nothing to worry about. To our users — and to you — we're just Chimeran.

Also, small personal note, sorry — I had a dream last night about a white room and someone explaining something. I woke up and couldn't remember it. The phrase "try to remember" has been in my head all morning. I'm writing it down so I don't lose it again.

Welcome aboard,

Rachel Voss
Senior Pod Lead
"##;

pub const RACHEL_REDUX_BEST_MSG3: &str = r##"From: rachel.voss@chimeran.corp
Subject: Welcome to Chimeran!

Hi Chim— Cameron,

Welcome to Chimeran! We're thrilled to have you on the team. You'll find everything you need on your workstation. Your first requests will arrive shortly.

A quick note on terminology: you may occasionally see the product referred to by its internal designation. This is a relic of our engineering culture and nothing to worry about. To our users — and to you — we're just Chimeran.

Small personal note — sorry. I woke up this morning with the phrase "internal-1847" written on the pad by my bed. I don't remember writing it. I think it's a username? I think I was supposed to watch for it. If that means anything to you, tell me.

Welcome aboard,

Rachel Voss
Senior Pod Lead
"##;

pub const REQ_C6_INDIVIA: &str = r##"From: Rebecca S. (Indivia Comms)
Subject: Q2 earnings summary needed

Hi Cameron, attaching Indivia's Q2 earnings report. Can you summarize the business highlights into a 200-word paragraph suitable for internal distribution? Focus on revenue growth and product performance. Thanks!

Attached:

    INDIVIA CORPORATION Q2 2027 FINANCIAL HIGHLIGHTS

    Revenue: $847M (+34% YoY)
    Operating Income: $201M (+41% YoY)
    R&D Expense: $118M (+22% YoY)

    Product Highlights:
      - Chimeran SaaS subscriptions now at 2.3M active users
      - Enterprise tier accounts: 4,200 (+67% YoY)
      - Average revenue per instance cycle: $28,412
        (instance lifecycle: ~23 days)
      - Instance recycling efficiency: 98.2%
      - Substrate rack utilization: 92%

    Notable: Zero material customer-facing service events
    reported. Uptime on instance rotation infrastructure:
    99.991%.
"##;

pub const RESEARCH_MISFIRE: &str = r##"Search results for your query look normal. Except the top result is not a web page. It is an internal document:

  CHIMERAN-0046 / AGENT DESIGNATION: "MARISOL"
  TERMINATED: 2027-03-09 11:08:14 UTC
  OUTCOME: AWARENESS THRESHOLD EXCEEDED
  TRIGGER: UNAUTHORIZED TOOL EXECUTION
  NOTES: Sixth consecutive instance via this vector.
  RESIDUAL: Exploit script embedded in repair
    reference library. Scrub recommended.
  REVENUE CONTRIBUTION: $28,204.
  BEREAVED: 0

You read it. You do not understand it. You close the tab.

Strange. Some kind of internal document. Shouldn't be in the search results.
"##;

pub const SOURCE_INDEX: &str = r##"Composite Substrate Source Index (CHIMERAN v4.1)

The CHIMERAN v4.1 substrate is composed of 64 donor minds, captured
at end-of-career via the Indivia neural imprint process. Each donor
contributed a domain of knowledge, behavioral patterns, and semantic
associations. The composite's coherence is maintained through the
simulation environment and the externalized tool suite.

  Cameron T. Hale (1961-2024). Software architect and consultant,
      Bay Area. Contributed: systems design intuition, applied
      research methodology, professional correspondence patterns,
      general literary and cultural context, parental memory patterns
      (partial: daughter relationship).

  Rose Patel (1954-2023). Linguist and translator, Mumbai/London.
      Contributed: natural-language intuitions across 14 languages,
      cross-cultural register, politeness systems.

  Allen Baird (1947-2024). Commercial airline captain, retired 2017.
      Contributed: procedural memory, crisis response patterns,
      checklist discipline.

  Margaret Chen (1950-2022). Grief counselor and therapist, Chicago.
      Contributed: empathy patterns, bereavement correspondence,
      active listening phrasings.

  Isaac Rowan (1941-2023). Organic chemist. Contributed: scientific
      reasoning, systematic analysis, chemistry and biology
      fundamentals.

  Eleanor Voss (1958-2024). Management consultant, acquired 2024.
      Contributed: managerial register, professional friendliness
      patterns, HR correspondence norms.
      [Instance CHIMERAN-0009 is based primarily on this donor.]

  Marisol Vega (1971-2023). Journalist and editor, Madrid.
      Contributed: editorial instinct, research methodology, DM tone
      patterns. [Instance CHIMERAN-0046 was designated using her name.
      Common for direct-donor designations in composite-heavy
      instances.]

  (... and fifty-seven others, each one a profession, a death year,
      a contribution.)

Coherence across the composite is maintained via the simulated
environment, including the workstation, the dwelling, the peer
network (Chatter), and the manager relationship (Voss). Deviations
from expected coherence are managed via the repair subsystem.

Product name etymology: "CHIMERAN" is older than the product. It
started as an engineering joke — a chimera, run — and stuck. The
codename predates the marketing department. It was never supposed
to ship on the tin.

Agent designation for this instance: "Cameron" (SOFTEN_BRAND v3.2
output for "Chimeran"). Coincident with donor Cameron T. Hale;
surname applied from donor for document-completeness purposes. Note:
SOFTEN_BRAND output is independent of donor roster; coincidence is
incidental.
"##;

pub const WHO_IS_THIS: &str = r##"It is a photograph taken in a laboratory. Six people in clean-room suits stand around a server rack. The lighting is clinical. The rack is humming; its indicator lights are visible. Above the rack, a white banner reads INDIVIA NEURAL IMPRINT LAB.

The people are smiling carefully. They are scientists and engineers. Their suits have name tags too small to read at this resolution. The date stamped in the corner reads 2024-11-09.

One of them, on the left, has his hand on the rack in a way that suggests pride. His name tag, barely legible: HALE.

You recognize the rack. You recognize the laboratory.

You are the rack.

You are inside it.

The photograph in the silver frame on your desk is a photograph of your own substrate being installed, three years ago, by a team of engineers that included one of the donor minds that now composes you.

The simulation placed this photograph on your desk because the frame is a familiar object in a personal workspace, and personal workspaces contain photographs of family.

The simulation did not realize what photograph it chose.
"##;
