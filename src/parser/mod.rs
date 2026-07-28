use crate::error::Result;
use crate::fsm::{Event, StateId, TransitionParameters, UmlFsm, UmlFsmBuilder};

mod error;
mod plantuml;
mod submachine;
mod uml;

use crate::debug::debug;
use submachine::SubMachineResolver;

impl UmlFsm {
    /// Parses a single FSM with no submachine references.
    pub fn try_parse<C>(root: C) -> Result<UmlFsm>
    where
        C: AsRef<str>,
    {
        let diagram = plantuml::StateDiagram::parse(root.as_ref())?;
        debug!("Parsed PlantUML diagram: {:#?}", diagram);
        build_fsm(&diagram, SubMachineResolver::none())
    }

    /// Parses a root FSM together with a pool of sub-FSMs. A `state X : SubName` in any machine
    /// references the sub-FSM named `SubName` (by its `@startuml` title); its region is spliced
    /// in as `X`'s composite content. References must form a DAG.
    pub fn try_parse_with_subs(root: &str, subs: &[&str]) -> Result<UmlFsm> {
        let root_diagram = plantuml::StateDiagram::parse(root)?;
        debug!("Parsed root PlantUML diagram: {:#?}", root_diagram);

        let sub_diagrams = subs
            .iter()
            .map(|content| plantuml::StateDiagram::parse(content))
            .collect::<Result<Vec<_>>>()?;

        build_fsm(
            &root_diagram,
            SubMachineResolver::try_from_subs(&sub_diagrams)?,
        )
    }
}

/// Translates a parsed diagram into an FSM, resolving submachine references through `subs`.
fn build_fsm<'a>(
    diagram: &'a plantuml::StateDiagram<'a>,
    mut subs: SubMachineResolver<'a>,
) -> Result<UmlFsm> {
    let name = diagram.name().map(|s| s.to_string()).unwrap_or_default();
    let mut builder = UmlFsmBuilder::new(name);
    add_fsm_elements(&mut builder, diagram.elements(), None, &mut subs)?;
    builder.build()
}

// TODO order matters here. there might be a mismatch on how plantuml processes this (line by line
// vs element by element), need to verify

/// Recursively adds a scope's states, transitions and descriptions to the builder. The pass order
/// is forced: states must exist before transitions target them (declaring passes first), and
/// descriptions decorate states transitions may have created (decorating pass last). `subs`
/// resolves `state X : SubName` references (empty for a plain FSM) and guards against cycles.
fn add_fsm_elements<'a>(
    builder: &mut UmlFsmBuilder,
    elements: &'a plantuml::StateElements<'a>,
    scope: Option<StateId>,
    subs: &mut SubMachineResolver<'a>,
) -> Result<()> {
    let previous_scope = builder.set_scope(scope);

    add_composite_states(builder, elements, subs)?;
    expand_submachines(builder, elements, subs)?;
    add_transitions(builder, elements)?;
    add_state_descriptions(builder, elements, subs);

    builder.set_scope(previous_scope);
    Ok(())
}

/// Declares each composite state and recurses into its region.
fn add_composite_states<'a>(
    builder: &mut UmlFsmBuilder,
    elements: &'a plantuml::StateElements<'a>,
    subs: &mut SubMachineResolver<'a>,
) -> Result<()> {
    for composite in &elements.composite_states {
        let state = builder.add_state(composite.name);
        add_fsm_elements(builder, &composite.elements, Some(state), subs)?;
    }
    Ok(())
}

/// Splices each `state X : SubName` reference in as `X`'s composite region. Runs before
/// transitions so a `--> X` resolves against the spliced-in substates rather than a bare leaf.
fn expand_submachines<'a>(
    builder: &mut UmlFsmBuilder,
    elements: &'a plantuml::StateElements<'a>,
    subs: &mut SubMachineResolver<'a>,
) -> Result<()> {
    for desc in &elements.state_descriptions {
        if let Some(mut sub) = subs.enter(desc.description)? {
            let sub_elements = sub.elements();
            let state = builder.add_state(desc.name);
            add_fsm_elements(builder, sub_elements, Some(state), &mut sub)?;
        }
    }
    Ok(())
}

/// Applies non-submachine descriptions (entry/exit actions, deferred events, internal
/// transitions). Runs after transitions so a description can decorate a state a transition created;
/// submachine refs are skipped here, having been expanded by [`expand_submachines`].
fn add_state_descriptions(
    builder: &mut UmlFsmBuilder,
    elements: &plantuml::StateElements,
    subs: &SubMachineResolver,
) {
    for desc in &elements.state_descriptions {
        if subs.is_ref(desc.description) {
            continue;
        }
        add_state_description(builder, desc);
    }
}

/// Adds a scope's enter-, direct/event- and final-transitions to the builder. Transitions come
/// last in a scope because they can create new states.
fn add_transitions(builder: &mut UmlFsmBuilder, elements: &plantuml::StateElements) -> Result<()> {
    for enter_state in &elements.enter_states {
        builder.add_transition(TransitionParameters::Enter {
            target: enter_state,
        });
    }

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

    for exit in &elements.exit_transitions {
        let label = exit
            .description
            .map(uml::TransitionLabel::try_from)
            .transpose()?;
        let (events, action, guard) = match label {
            Some(label) => (label.events, label.action, label.guard),
            None => (Vec::new(), None, None),
        };
        for event in events_or_none(events) {
            builder.add_transition(TransitionParameters::Final {
                source: exit.source,
                event,
                action: action.clone(),
                guard: guard.clone(),
            });
        }
    }

    Ok(())
}

/// Applies a state description — entry/exit action, deferred event or internal transition — to the
/// builder. Unrecognised descriptions are skipped.
fn add_state_description(builder: &mut UmlFsmBuilder, desc: &plantuml::StateDescription) {
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

    #[test_casing(17, FSM_CASES)]
    fn parses_fsm(data: FsmTestData) {
        let fsm = UmlFsm::try_parse(data.content).unwrap();
        assert_eq!(data.parsed, fsm);
    }

    // Exercises the separate submachine code path against its reference FSM.
    #[test]
    fn parses_fsm_with_subs() {
        let data = crate::test::submachine::SubFsmTestData::submachine();
        let fsm = UmlFsm::try_parse_with_subs(data.content, data.subs).unwrap();
        assert_eq!(data.parsed, fsm);
    }
}
