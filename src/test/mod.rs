use std::path::PathBuf;

use crate::fsm;
mod actions;
mod composite_states;
mod deferred_events;
mod exit_states;
mod four_seasons;
mod misc;
mod transitions;
mod utils;

pub struct FsmTestData {
    pub name: &'static str,
    pub content: &'static str,
    pub parsed: fsm::UmlFsm,
    pub path: PathBuf,
}

impl std::fmt::Debug for FsmTestData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FsmTestData")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("parsed", &self.parsed)
            .finish()
    }
}

impl FsmTestData {
    pub fn all() -> impl Iterator<Item = Self> {
        // TODO lazy iter
        vec![
            Self::actions(),
            Self::composite_states(),
            Self::enter_exit(),
            Self::exit_states(),
            Self::four_seasons(),
            Self::misc(),
            Self::guards(),
            Self::internal_transitions(),
            Self::same_name_substates(),
            Self::substate_to_substate(),
            Self::transitions(),
            Self::direct_transitions(),
            Self::deferred_events(),
        ]
        .into_iter()
    }
}
