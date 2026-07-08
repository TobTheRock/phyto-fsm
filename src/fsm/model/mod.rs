mod fsm;
mod state;
mod transition;

pub type StateId = indextree::NodeId;

pub use fsm::UmlFsm;
pub use state::{State, StateData};
pub use transition::{Target, TargetData, TransitionData, TransitionParameters, TransitionTarget};
