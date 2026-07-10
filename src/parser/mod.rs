use crate::error::{Error, Result};
use crate::fsm::{Event, StateId, TransitionParameters, UmlFsm, UmlFsmBuilder};

mod error;
mod plantuml;
mod uml;

use crate::debug::debug;

impl UmlFsm {
    pub fn try_parse<C>(content: C) -> Result<UmlFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(content.as_ref())?;
        debug!("Parsed PlantUML diagram: {:#?}", diagram);
        diagram.try_into()
    }
}

impl TryFrom<plantuml::StateDiagram<'_>> for UmlFsm {
    type Error = Error;
    fn try_from(diagram: plantuml::StateDiagram<'_>) -> Result<Self> {
        let name = diagram.name().map(|s| s.to_string()).unwrap_or_default();
        let mut builder = UmlFsmBuilder::new(name);

        add_fsm_elements(&mut builder, diagram.elements(), None)?;

        builder.build()
    }
}

// TODO order matters here. there might be a mismatch on how plantuml processes this (line by line
// vs element by element), need to verify
fn add_fsm_elements(
    builder: &mut UmlFsmBuilder,
    elements: &plantuml::StateElements<'_>,
    scope: Option<StateId>,
) -> Result<()> {
    let previous_scope = builder.set_scope(scope);

    for composite in &elements.composite_states {
        let state = builder.add_state(composite.name);
        add_fsm_elements(builder, &composite.elements, Some(state))?;
    }

    for enter_state in &elements.enter_states {
        builder.add_transition(TransitionParameters::Enter {
            target: enter_state,
        });
    }
    // Add transitions last, as they can create new states
    for transition in &elements.transitions {
        let label = transition
            .description
            .map(uml::TransitionLabel::try_from)
            .transpose()?;
        let (events, action, guard) = match label {
            Some(label) => (label.events, label.action, label.guard),
            None => (Vec::new(), None, None),
        };
        // An event list desugars to one transition per event; with no events it is a single
        // event-less (direct) transition.
        for event in events_or_none(events) {
            let params = match event {
                Some(event) => TransitionParameters::Event {
                    source: transition.source,
                    target: transition.target,
                    event,
                    action: action.clone(),
                    guard: guard.clone(),
                },
                None => TransitionParameters::Direct {
                    source: transition.source,
                    target: transition.target,
                    action: action.clone(),
                    guard: guard.clone(),
                },
            };
            builder.add_transition(params);
        }
    }

    for desc in &elements.state_descriptions {
        match uml::StateDescription::try_from(desc.description) {
            Ok(uml::StateDescription::Entry(action)) => {
                builder.add_enter_action(desc.name, action);
            }
            Ok(uml::StateDescription::Exit(action)) => {
                builder.add_exit_action(desc.name, action);
            }
            Ok(uml::StateDescription::DeferEvent(event)) => {
                builder.add_deferred_event(desc.name, event);
            }
            Ok(uml::StateDescription::InternalTransition(label)) => {
                // An internal transition always carries an event (`State : Event / action`).
                for event in label.events {
                    builder.add_transition(TransitionParameters::Internal {
                        source: desc.name,
                        event,
                        action: label.action.clone(),
                        guard: label.guard.clone(),
                    });
                }
            }
            Err(_) => {} // unrecognised description, skip
        }
    }

    builder.set_scope(previous_scope);
    Ok(())
}

/// Yields each event as `Some`, or a single `None` when the list is empty (a direct,
/// event-less transition). Lets event-list desugaring and direct transitions share one loop.
fn events_or_none(events: Vec<Event>) -> std::vec::IntoIter<Option<Event>> {
    if events.is_empty() {
        vec![None].into_iter()
    } else {
        events.into_iter().map(Some).collect::<Vec<_>>().into_iter()
    }
}

#[cfg(test)]
mod test {
    use crate::{fsm::UmlFsm, test::FsmTestData};
    use pretty_assertions::assert_eq;
    use test_casing::{TestCases, cases, test_casing};

    const FSM_CASES: TestCases<FsmTestData> = cases!(FsmTestData::all());

    #[test_casing(12, FSM_CASES)]
    fn parses_fsm(data: FsmTestData) {
        let fsm = UmlFsm::try_parse(data.content).unwrap();
        assert_eq!(data.parsed, fsm);
    }
}
