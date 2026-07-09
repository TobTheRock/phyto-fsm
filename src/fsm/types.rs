use derive_more::{Display, From, Into};

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Event(pub String);

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, From, Into)]
pub struct Action(pub String);

impl From<&str> for Event {
    fn from(s: &str) -> Self {
        Event(s.to_string())
    }
}

impl From<&str> for Action {
    fn from(s: &str) -> Self {
        Action(s.to_string())
    }
}
