use crate::error::{Error, Result};
use crate::fsm::{StateId, StateType, TransitionParameters, UmlFsm, UmlFsmBuilder};

mod error;
mod plantuml;
mod uml;

use log::trace;

impl UmlFsm {
    pub fn try_parse<C>(content: C) -> Result<UmlFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(content.as_ref())?;
        trace!("Parsed PlantUML diagram: {:#?}", diagram);
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
        let state = builder.add_state(composite.name, StateType::Simple);
        add_fsm_elements(builder, &composite.elements, Some(state))?;
    }

    for enter_state in &elements.enter_states {
        builder.add_state(enter_state, StateType::Enter);
    }
    // Add transitions last, as they can create new states
    for transition in &elements.transitions {
        let (event, action, guard) = if let Some(desc) = transition.description {
            let label = uml::TransitionLabel::try_from(desc)?;
            (label.event, label.action, label.guard)
        } else {
            (None, None, None)
        };
        builder.add_transition(TransitionParameters {
            source: transition.source,
            target: Some(transition.target),
            event,
            action,
            guard,
        });
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
                builder.add_transition(TransitionParameters {
                    source: desc.name,
                    target: None,
                    event: label.event,
                    action: label.action,
                    guard: label.guard,
                });
            }
            Err(_) => {} // unrecognised description, skip
        }
    }

    builder.set_scope(previous_scope);
    Ok(())
}

impl<'a> TryFrom<plantuml::TransitionDescription<'a>> for TransitionParameters<'a> {
    type Error = crate::error::Error;
    fn try_from(transition: plantuml::TransitionDescription<'a>) -> Result<Self> {
        let (event, action, guard) = if let Some(desc) = transition.description {
            let label = uml::TransitionLabel::try_from(desc)?;
            (label.event, label.action, label.guard)
        } else {
            (None, None, None)
        };
        Ok(Self {
            source: transition.source,
            target: Some(transition.target),
            event,
            action,
            guard,
        })
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
        crate::logging::init();
        let fsm = UmlFsm::try_parse(data.content).unwrap();
        assert_eq!(data.parsed, fsm);
    }
}
