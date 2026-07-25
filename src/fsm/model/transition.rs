use crate::fsm::types::{Action, Event};

use super::StateId;
use super::state::{CompletionTransition, State, StateData};

#[derive(Debug, Clone)]
pub enum TransitionKind<S, E, A> {
    /// `S -- event [guard] / action --> target`: runs exit(S) + enter(target).
    Event {
        source: S,
        event: E,
        target: S,
        action: Option<A>,
        guard: Option<A>,
    },
    /// `S : event [guard] / action`: runs action, no state change, no exit/enter.
    Internal {
        source: S,
        event: E,
        action: Option<A>,
        guard: Option<A>,
    },
    /// `S -- [guard] / action --> target`: completion transition, auto-fires.
    Direct {
        source: S,
        target: S,
        action: Option<A>,
        guard: Option<A>,
    },
    /// `[*] --> target`: the scope's initial pseudo-transition. Owned by `target`,
    /// which marks it as the state entered when its scope becomes active.
    Enter { target: S },
    /// `S -- event [guard] / action --> [*]`: a transition to the `[*]` final pseudo-state.
    /// Reaching it ends the enclosing region — the whole FSM at top level. `event: None` is a
    /// completion (event-less) exit.
    Final {
        source: S,
        event: Option<E>,
        action: Option<A>,
        guard: Option<A>,
    },
}

/// Source-level transition input: endpoints are state *names*, resolved to ids by the builder.
/// One value per transition kind — the builder no longer classifies from optional fields.
pub type TransitionParameters<'a> = TransitionKind<&'a str, Event, Action>;

/// Arena-stored transition: owns its event/action and refers to states by id.
pub type TransitionData = TransitionKind<StateId, Event, Action>;

/// Resolved view over a [`TransitionData`], with states looked up in the arena.
pub type Transition<'a> = TransitionKind<State<'a>, &'a Event, &'a Action>;

impl TransitionData {
    pub fn source(&self) -> StateId {
        match self {
            TransitionData::Event { source, .. }
            | TransitionData::Internal { source, .. }
            | TransitionData::Direct { source, .. }
            | TransitionData::Final { source, .. } => *source,
            TransitionData::Enter { target } => *target,
        }
    }

    pub fn event(&self) -> Option<&Event> {
        match self {
            TransitionData::Event { event, .. } | TransitionData::Internal { event, .. } => {
                Some(event)
            }
            TransitionData::Final { event, .. } => event.as_ref(),
            TransitionData::Direct { .. } | TransitionData::Enter { .. } => None,
        }
    }

    pub fn action(&self) -> Option<&Action> {
        match self {
            TransitionData::Event { action, .. }
            | TransitionData::Internal { action, .. }
            | TransitionData::Direct { action, .. }
            | TransitionData::Final { action, .. } => action.as_ref(),
            TransitionData::Enter { .. } => None,
        }
    }

    pub fn guard(&self) -> Option<&Action> {
        match self {
            TransitionData::Event { guard, .. }
            | TransitionData::Internal { guard, .. }
            | TransitionData::Direct { guard, .. }
            | TransitionData::Final { guard, .. } => guard.as_ref(),
            TransitionData::Enter { .. } => None,
        }
    }
}

impl CompletionTransition<'_> {
    /// Redirects this parent completion onto a substate exit: keeps the exit's `source` and trigger
    /// while adopting the completion's target/effect/guard. `event: Some` yields an event-driven
    /// `Event`, a bare completion (`event: None`) yields a `Direct`.
    pub fn redirect(self, source: StateId, event: Option<Event>) -> TransitionData {
        let CompletionTransition {
            target,
            action,
            guard,
        } = self;
        match event {
            Some(event) => TransitionData::Event {
                source,
                event,
                target,
                action: action.cloned(),
                guard: guard.cloned(),
            },
            None => TransitionData::Direct {
                source,
                target,
                action: action.cloned(),
                guard: guard.cloned(),
            },
        }
    }
}

impl<'a> Transition<'a> {
    pub fn from(
        data: &'a TransitionData,
        arena: &'a indextree::Arena<StateData>,
    ) -> Transition<'a> {
        match data {
            TransitionData::Event {
                source,
                event,
                target,
                action,
                guard,
            } => Transition::Event {
                source: State::new(*source, arena),
                event,
                target: State::new(*target, arena),
                action: action.as_ref(),
                guard: guard.as_ref(),
            },
            TransitionData::Internal {
                source,
                event,
                action,
                guard,
            } => Transition::Internal {
                source: State::new(*source, arena),
                event,
                action: action.as_ref(),
                guard: guard.as_ref(),
            },
            TransitionData::Direct {
                source,
                target,
                action,
                guard,
            } => Transition::Direct {
                source: State::new(*source, arena),
                target: State::new(*target, arena),
                action: action.as_ref(),
                guard: guard.as_ref(),
            },
            TransitionData::Enter { target } => Transition::Enter {
                target: State::new(*target, arena),
            },
            TransitionData::Final {
                source,
                event,
                action,
                guard,
            } => Transition::Final {
                source: State::new(*source, arena),
                event: event.as_ref(),
                action: action.as_ref(),
                guard: guard.as_ref(),
            },
        }
    }

    pub fn source(&self) -> State<'a> {
        match self {
            Transition::Event { source, .. }
            | Transition::Internal { source, .. }
            | Transition::Direct { source, .. }
            | Transition::Final { source, .. } => *source,
            Transition::Enter { target } => *target,
        }
    }

    pub fn event(&self) -> Option<&'a Event> {
        match self {
            Transition::Event { event, .. } | Transition::Internal { event, .. } => Some(event),
            Transition::Final { event, .. } => *event,
            Transition::Direct { .. } | Transition::Enter { .. } => None,
        }
    }

    /// `None` for internal and final transitions, which have no target state.
    pub fn destination(&self) -> Option<State<'a>> {
        match self {
            Transition::Event { target, .. } | Transition::Direct { target, .. } => Some(*target),
            Transition::Internal { .. } | Transition::Enter { .. } | Transition::Final { .. } => {
                None
            }
        }
    }

    pub fn action(&self) -> Option<&'a Action> {
        match self {
            Transition::Event { action, .. }
            | Transition::Internal { action, .. }
            | Transition::Direct { action, .. }
            | Transition::Final { action, .. } => *action,
            Transition::Enter { .. } => None,
        }
    }

    pub fn guard(&self) -> Option<&'a Action> {
        match self {
            Transition::Event { guard, .. }
            | Transition::Internal { guard, .. }
            | Transition::Direct { guard, .. }
            | Transition::Final { guard, .. } => *guard,
            Transition::Enter { .. } => None,
        }
    }
}

impl std::fmt::Display for Transition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_name = self.event().map(|e| e.0.as_str()).unwrap_or("(direct)");
        let guard = self
            .guard()
            .map(|g| format!(" [{}]", g.0))
            .unwrap_or_default();
        let action = self
            .action()
            .map(|a| format!(" / {}", a.0))
            .unwrap_or_default();
        let dest = self.destination();
        let dest = match self {
            Transition::Final { .. } => "[*]",
            _ => dest.as_ref().map(|d| d.name()).unwrap_or("(internal)"),
        };
        write!(
            f,
            "{} --[{}{}{}]--> {}",
            self.source().name(),
            event_name,
            guard,
            action,
            dest
        )
    }
}

impl PartialEq for Transition<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.source().name() == other.source().name() && self.event() == other.event()
    }
}

impl Eq for Transition<'_> {}

impl PartialOrd for Transition<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transition<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source()
            .name()
            .cmp(other.source().name())
            .then_with(|| {
                let self_event = self.event().map(|e| e.0.as_str()).unwrap_or("");
                let other_event = other.event().map(|e| e.0.as_str()).unwrap_or("");
                self_event.cmp(other_event)
            })
    }
}
