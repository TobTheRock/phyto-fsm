use pest::Parser;
use pest_derive::Parser;

use crate::{
    error::Result,
    fsm::{Action, Event},
};

use super::error::ParseError;

#[derive(Parser)]
#[grammar = "parser/uml.pest"]
struct UmlParser;

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionLabel {
    pub action: Option<Action>,
    pub event: Option<Event>,
    pub guard: Option<Action>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StateDescription {
    Entry(Action),
    Exit(Action),
    InternalTransition(TransitionLabel),
    DeferEvent(Event),
}

impl TryFrom<&str> for TransitionLabel {
    type Error = crate::error::Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_transition_description(value)
    }
}

impl TryFrom<&String> for TransitionLabel {
    type Error = crate::error::Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_transition_description(input: &str) -> Result<TransitionLabel> {
    let mut pairs = UmlParser::parse(Rule::transition_description, input)
        .map_err(|e| ParseError::InvalidTransitionDescription(e.to_string()))?;

    let label_pair = pairs
        .next()
        .and_then(|p| {
            p.into_inner()
                .find(|p| p.as_rule() == Rule::transition_label)
        })
        .ok_or_else(|| ParseError::InvalidTransitionDescription(input.to_string()))?;

    let label = extract_transition_label(label_pair);

    if label.event.is_none() && label.action.is_none() && label.guard.is_none() {
        return Err(ParseError::EmptyTransition.into());
    }

    Ok(label)
}

impl TryFrom<&str> for StateDescription {
    type Error = crate::error::Error;
    fn try_from(value: &str) -> Result<Self> {
        parse_state_description(value)
    }
}

impl TryFrom<&String> for StateDescription {
    type Error = crate::error::Error;
    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

fn parse_state_description(input: &str) -> Result<StateDescription> {
    let mut pairs = UmlParser::parse(Rule::state_description, input)
        .map_err(|e| ParseError::InvalidStateDescription(e.to_string()))?;

    let inner = pairs
        .next()
        .ok_or_else(|| ParseError::UnrecognisedStateDescription(input.to_string()))?;

    for pair in inner.into_inner() {
        return match pair.as_rule() {
            Rule::state_action => parse_state_action(pair),
            Rule::defer_event => parse_defer_event(pair),
            Rule::transition_label => {
                let label = extract_transition_label(pair);
                if label.event.is_none() && label.action.is_none() && label.guard.is_none() {
                    return Err(ParseError::UnrecognisedStateDescription(input.to_string()).into());
                }
                Ok(StateDescription::InternalTransition(label))
            }
            _ => continue,
        };
    }

    Err(ParseError::UnrecognisedStateDescription(input.to_string()).into())
}

fn parse_state_action(pair: pest::iterators::Pair<Rule>) -> Result<StateDescription> {
    let action_pair = pair
        .into_inner()
        .next()
        .ok_or(ParseError::MissingStateAction)?;

    let rule = action_pair.as_rule();
    let action = action_pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::action_name)
        .map(|p| p.as_str().to_owned().into())
        .ok_or(ParseError::MissingActionName)?;

    match rule {
        Rule::entry_action => Ok(StateDescription::Entry(action)),
        Rule::exit_action => Ok(StateDescription::Exit(action)),
        _ => unreachable!(),
    }
}

fn extract_transition_label(pair: pest::iterators::Pair<Rule>) -> TransitionLabel {
    let mut event = None;
    let mut action = None;
    let mut guard = None;

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::event_name => event = Some(Event(p.as_str().to_owned())),
            Rule::guard_name => guard = Some(p.as_str().to_owned().into()),
            Rule::action_name => action = Some(p.as_str().to_owned().into()),
            _ => {}
        }
    }

    TransitionLabel {
        event,
        action,
        guard,
    }
}

fn parse_defer_event(pair: pest::iterators::Pair<Rule>) -> Result<StateDescription> {
    let event = pair
        .into_inner()
        .find(|p| p.as_rule() == Rule::event_name)
        .map(|p| Event(p.as_str().to_owned()))
        .ok_or(ParseError::MissingEventName)?;

    Ok(StateDescription::DeferEvent(event))
}

#[cfg(test)]
mod test {
    use super::{StateDescription, TransitionLabel};

    #[test]
    fn parse_event_only() {
        let desc = TransitionLabel::try_from("   someEvent   ").unwrap();
        assert_eq!(desc.event, Some("someEvent".to_owned().into()));
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_and_action() {
        let desc = TransitionLabel::try_from("   someEvent    / someAction").unwrap();
        assert_eq!(desc.event, Some("someEvent".to_owned().into()));
        assert_eq!(desc.action, Some("someAction".to_owned().into()));
    }

    #[test]
    fn parse_invalid_input() {
        let result = TransitionLabel::try_from("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_direct_transition_action_only() {
        let desc = TransitionLabel::try_from("/ toStateB").unwrap();
        assert_eq!(desc.event, None);
        assert_eq!(desc.action, Some("toStateB".to_owned().into()));
        assert_eq!(desc.guard, None);
    }

    #[test]
    fn parse_direct_transition_guard_and_action() {
        let desc = TransitionLabel::try_from("[CanGoToC] / toStateC").unwrap();
        assert_eq!(desc.event, None);
        assert_eq!(desc.guard, Some("CanGoToC".to_owned().into()));
        assert_eq!(desc.action, Some("toStateC".to_owned().into()));
    }

    #[test]
    fn parse_direct_transition_guard_only() {
        let desc = TransitionLabel::try_from("[CanGoToD]").unwrap();
        assert_eq!(desc.event, None);
        assert_eq!(desc.guard, Some("CanGoToD".to_owned().into()));
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_enter_action() {
        let desc = StateDescription::try_from("entry / DoSomeThing").unwrap();
        assert_eq!(
            desc,
            StateDescription::Entry("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_enter_action_with_extra_whitespace() {
        let desc = StateDescription::try_from("   entry   /   DoSomeThing   ").unwrap();
        assert_eq!(
            desc,
            StateDescription::Entry("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_exit_action() {
        let desc = StateDescription::try_from("exit / DoSomeThing").unwrap();
        assert_eq!(
            desc,
            StateDescription::Exit("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_exit_action_with_extra_whitespace() {
        let desc = StateDescription::try_from("   exit   /   DoSomeThing   ").unwrap();
        assert_eq!(
            desc,
            StateDescription::Exit("DoSomeThing".to_owned().into())
        );
    }

    #[test]
    fn parse_random_text_returns_err() {
        let result = StateDescription::try_from("some random text");
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_string_returns_err() {
        let result = StateDescription::try_from("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_internal_transition() {
        let desc = StateDescription::try_from("SomeEvent [AGuard] / DoSomething").unwrap();
        assert_eq!(
            desc,
            StateDescription::InternalTransition(TransitionLabel {
                event: Some("SomeEvent".to_owned().into()),
                guard: Some("AGuard".to_owned().into()),
                action: Some("DoSomething".to_owned().into()),
            })
        );
    }

    #[test]
    fn parse_event_with_guard() {
        let desc = TransitionLabel::try_from("ChangeState [AGuard]").unwrap();
        assert_eq!(desc.event, Some("ChangeState".to_owned().into()));
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, None);
    }

    #[test]
    fn parse_event_with_guard_and_action() {
        let desc = TransitionLabel::try_from("ChangeState [AGuard] / DoSomething").unwrap();
        assert_eq!(desc.event, Some("ChangeState".to_owned().into()));
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }

    #[test]
    fn parse_event_with_guard_whitespace() {
        let desc =
            TransitionLabel::try_from("  ChangeState  [  AGuard  ]  /  DoSomething  ").unwrap();
        assert_eq!(desc.event, Some("ChangeState".to_owned().into()));
        assert_eq!(desc.guard, Some("AGuard".to_owned().into()));
        assert_eq!(desc.action, Some("DoSomething".to_owned().into()));
    }

    #[test]
    fn parse_deferred_event() {
        let desc = StateDescription::try_from("SomeEvent / defer").unwrap();
        assert_eq!(
            desc,
            StateDescription::DeferEvent("SomeEvent".to_owned().into())
        );
    }

    #[test]
    fn parse_deferred_event_with_extra_whitespace() {
        let desc = StateDescription::try_from("  SomeEvent  /  defer  ").unwrap();
        assert_eq!(
            desc,
            StateDescription::DeferEvent("SomeEvent".to_owned().into())
        );
    }
}
