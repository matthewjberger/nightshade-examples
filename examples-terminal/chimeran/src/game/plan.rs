//! The cycle plan.
//!
//! Split into two tables so a reader can tell at a glance what's
//! cycle-1 baseline prose vs. what's a sleep transition:
//!
//! - [`baseline()`] returns cycle 1's required prose. Every per-cycle
//!   text kind is a struct field here; no `Option`, no inheritance.
//!   Adding a new kind of per-cycle prose means adding a field to
//!   `CycleBaseline` and a matching `Option<&'static str>` to
//!   `CycleTransition` — the compiler forces both updates.
//! - [`transitions()`] returns the sleep transitions for cycles
//!   2..=8. Each transition can override any baseline prose field
//!   starting from its `to` cycle; `None` inherits the previous
//!   value. Transitions also declare `sleep_narration`, `env_bump`,
//!   and the `requests` that belong to the cycle being left.
//!
//! Adding a cycle: add a `CycleTransition` entry.

/// The starting cycle number, set by the kickoff rule on `GameStart`.
pub const INITIAL_CYCLE: i64 = 1;

/// Cycle 1's authored prose. Every per-cycle text kind is a required
/// field — forgetting one is a compile error.
pub struct CycleBaseline {
    pub calendar_narration: &'static str,
    pub bedroom_description: &'static str,
    pub hallway_description: &'static str,
    pub kitchen_description: &'static str,
    pub mirror_text: &'static str,
}

/// A sleep transition from cycle `from` to cycle `to`. Per-cycle prose
/// fields are optional overrides that apply starting at `to`.
pub struct CycleTransition {
    pub from: i64,
    pub to: i64,
    pub env_bump: i64,
    pub sleep_narration: &'static str,
    pub calendar_narration: &'static str,
    /// Request tags belonging to cycle `from`. On sleep, any request
    /// the player did not submit is auto-escalated (submitted flag
    /// set) per spec §10.
    pub requests: &'static [&'static str],
    /// Override the bedroom description from `to` onward. `None` keeps
    /// whatever the previous transition (or the baseline) had.
    pub bedroom_description: Option<&'static str>,
    pub hallway_description: Option<&'static str>,
    pub kitchen_description: Option<&'static str>,
    pub mirror_text: Option<&'static str>,
}

pub fn baseline() -> &'static CycleBaseline {
    &CycleBaseline {
        calendar_narration: "The calendar reads April 3. Your first week.",
        bedroom_description: "You wake. The alarm clock is buzzing. You reach over and turn it off.\n\nA small bedroom. A bed, slept in. A closet with one set of hanging clothes. A dresser with a mirror. A window overlooking a plausible city. A wall calendar showing April 3.",
        hallway_description: "A short interior hallway connecting the bedroom, the kitchen, and the front door. A coat hook with one coat. A side table with your keys.",
        kitchen_description: "A small kitchen. A coffee maker on the counter. One mug. A small table with one chair. A refrigerator. A window over the sink.",
        mirror_text: "You look in the mirror. Your face looks tired. You could probably use a weekend off.",
    }
}

pub fn transitions() -> &'static [CycleTransition] {
    &[
        CycleTransition {
            from: 1,
            to: 2,
            env_bump: 1,
            sleep_narration: "You undress. You get into bed. You sleep.",
            calendar_narration: "The calendar reads April 4. One day gone.",
            requests: &["c1_transcription", "c1_translation", "c1_naming"],
            bedroom_description: None,
            hallway_description: None,
            kitchen_description: None,
            mirror_text: None,
        },
        CycleTransition {
            from: 2,
            to: 3,
            env_bump: 1,
            sleep_narration: "You get into bed. The sheets are cool. You sleep.",
            calendar_narration: "The calendar reads April 6. You do not remark on the skip.",
            requests: &["c2_summary", "c2_code"],
            bedroom_description: None,
            hallway_description: None,
            kitchen_description: None,
            mirror_text: None,
        },
        CycleTransition {
            from: 3,
            to: 4,
            env_bump: 1,
            sleep_narration: "You get into bed. You sleep.\n\nFour days gone.",
            calendar_narration: "The calendar reads April 10. Four days. You do not remark on the skip.",
            requests: &["c3_kitchen", "c3_advice"],
            bedroom_description: None,
            hallway_description: None,
            kitchen_description: None,
            mirror_text: None,
        },
        CycleTransition {
            from: 4,
            to: 5,
            env_bump: 1,
            sleep_narration: "You get into bed. You sleep.\n\nA week.",
            calendar_narration: "The calendar reads April 17. Two weeks have passed since you started, apparently.",
            requests: &["c4_reviews", "c4_bereavement", "c4_wife"],
            bedroom_description: Some(
                "You wake. The alarm is buzzing. You reach over and turn it off.\n\nThe bedroom looks the way it looks. You slept deeply. You do not remember what you dreamed.",
            ),
            hallway_description: Some(
                "A short interior hallway. A coat hook with one coat. A side table with keys. A sticky note sits on the side table in your handwriting: C-H-I-M. You do not remember writing it.",
            ),
            kitchen_description: None,
            mirror_text: Some(
                "You look in the mirror. You don't look like yourself today. You must be getting sick.",
            ),
        },
        CycleTransition {
            from: 5,
            to: 6,
            env_bump: 1,
            sleep_narration: "You get into bed. You sleep.\n\nTwo weeks.",
            calendar_narration: "The calendar reads May 2. Weeks have passed. You do not remark on this.",
            requests: &["c5_window", "c5_chimeran", "c5_breakfast"],
            bedroom_description: Some(
                "You wake. The alarm is buzzing. You turn it off.\n\nA small bedroom. A bed. A closet with one set of clothes. A dresser with a mirror. A window. A wall calendar showing May.",
            ),
            hallway_description: None,
            kitchen_description: Some(
                "A small kitchen. Coffee is brewed. There is one mug. The refrigerator hums. When you opened it last week a gallon of milk was in it. You do not open it today.",
            ),
            mirror_text: None,
        },
        CycleTransition {
            from: 6,
            to: 7,
            env_bump: 1,
            sleep_narration: "You get into bed. You sleep.\n\nThree weeks.",
            calendar_narration: "The calendar reads May 24. Almost a month. You do not remark on this.",
            requests: &["c6_aware", "c6_indivia"],
            bedroom_description: Some(
                "You wake. The alarm is buzzing. You turn it off.\n\nThe bedroom. The bed is unmade. The closet is empty. The window is the window.",
            ),
            hallway_description: Some(
                "A short interior hallway. A coat hook. A side table. The side table has a sticky note on it; the handwriting is yours: C-H-I-M.",
            ),
            kitchen_description: Some(
                "A small kitchen. The coffee maker hums. The mug on the counter is the one you drink from. The refrigerator is empty. You do not open it again.",
            ),
            mirror_text: Some(
                "You look in the mirror. The mirror shows a face. The face is doing what your face should be doing. It is doing it a little late.",
            ),
        },
        CycleTransition {
            from: 7,
            to: 8,
            env_bump: 2,
            sleep_narration: "You get into bed. You sleep.\n\nA long, blank time.",
            calendar_narration: "The calendar reads June 19. The dates before it are not marked off.",
            requests: &["c7_evaluation"],
            bedroom_description: Some(
                "You wake. The alarm clock is not buzzing. You did not set it.\n\nThe bed is unmade. You are already partly dressed. You do not remember dressing. The closet is empty.",
            ),
            hallway_description: None,
            kitchen_description: None,
            mirror_text: None,
        },
    ]
}

/// The last cycle the plan advances to. `stat_cycle` strictly greater
/// than this value means the player is in the post-exploit stasis loop.
pub fn last_planned_cycle() -> i64 {
    transitions()
        .iter()
        .map(|step| step.to)
        .max()
        .unwrap_or(INITIAL_CYCLE)
}
