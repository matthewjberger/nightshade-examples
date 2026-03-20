use stateless::statemachine;

statemachine! {
    name: TurnPhase,
    derive_states: [Debug, Copy, Clone, PartialEq, Eq, Hash],
    derive_events: [Debug, Copy, Clone, PartialEq, Eq],
    transitions: {
        *Reinforcement + SpawnsProcessed = Action,
        Action + ActionsExhausted | EndTurnPressed = End,
        End + TurnAdvanced = Reinforcement,
    }
}
