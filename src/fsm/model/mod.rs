mod fsm;
mod state;
pub mod transition;

pub type StateId = indextree::NodeId;

pub use fsm::UmlFsm;
pub use state::{State, StateData};
pub use transition::{Transition, TransitionData, TransitionParameters};
