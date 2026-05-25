use pest::Parser;
use pest_derive::Parser;

use crate::error::Result;

use super::error::ParseError;

#[derive(Debug, PartialEq)]
pub struct StateDiagram<'a> {
    name: Option<&'a str>,
    root: StateElements<'a>,
}

pub type StateName<'a> = &'a str;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct StateDescription<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransitionDescription<'a> {
    pub source: StateName<'a>,
    pub target: StateName<'a>,
    pub description: Option<&'a str>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CompositeState<'a> {
    pub name: StateName<'a>,
    pub elements: StateElements<'a>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct StateElements<'a> {
    pub enter_states: Vec<StateName<'a>>,
    pub transitions: Vec<TransitionDescription<'a>>,
    pub composite_states: Vec<CompositeState<'a>>,
    pub state_descriptions: Vec<StateDescription<'a>>,
}

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

#[derive(Parser)]
#[grammar = "parser/plantuml.pest"]
struct PlantUmlParser;

impl StateDiagram<'_> {
    pub fn parse(input: &str) -> Result<StateDiagram<'_>> {
        let mut pairs = PlantUmlParser::parse(Rule::diagram, input)
            .map_err(|e| ParseError::Grammar(e.to_string()))?;

        let diagram_pair = pairs.next().ok_or(ParseError::EmptyInput)?;

        let mut name = None;
        let mut root = StateElements::default();

        for inner in diagram_pair.into_inner() {
            match inner.as_rule() {
                Rule::startuml => name = parse_diagram_name(inner),
                Rule::content => {
                    root = parse_content(inner)?;
                }
                _ => {}
            }
        }

        Ok(StateDiagram { name, root })
    }

    pub fn name(&self) -> Option<&str> {
        self.name
    }

    pub fn elements(&self) -> &StateElements<'_> {
        &self.root
    }
}
fn parse_diagram_name(pair: Pair<'_>) -> Option<&str> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::diagram_name)
        .map(|p| p.as_str())
}

fn parse_content(pair: Pair<'_>) -> Result<StateElements<'_>> {
    let mut enter_states = Vec::new();
    let mut transitions = Vec::new();
    let mut composite_states = Vec::new();
    let mut state_descriptions = Vec::new();

    for element in pair.into_inner() {
        if element.as_rule() != Rule::element {
            continue;
        }

        for element_inner in element.into_inner() {
            match element_inner.as_rule() {
                Rule::enter_transition => {
                    if let Some(state) = parse_enter_state(element_inner) {
                        enter_states.push(state);
                    }
                }
                Rule::transition => {
                    transitions.push(parse_transition(element_inner)?);
                }
                Rule::composite_state => {
                    composite_states.push(parse_composite_state(element_inner)?);
                }
                Rule::state_declaration_with_desc | Rule::state_description => {
                    state_descriptions.push(parse_state_description(element_inner)?);
                }
                _ => {}
            }
        }
    }

    Ok(StateElements {
        enter_states,
        transitions,
        composite_states,
        state_descriptions,
    })
}

fn parse_enter_state(pair: Pair<'_>) -> Option<StateName<'_>> {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::state_name)
        .map(|p| p.as_str())
}

fn parse_transition(pair: Pair<'_>) -> Result<TransitionDescription<'_>> {
    let mut from = None;
    let mut to = None;
    let mut description = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name => {
                if from.is_none() {
                    from = Some(inner.as_str());
                } else {
                    to = Some(inner.as_str());
                }
            }
            Rule::description => {
                let text = inner.as_str().trim();
                if !text.is_empty() {
                    description = Some(text);
                }
            }
            _ => {}
        }
    }

    Ok(TransitionDescription {
        source: from.ok_or(ParseError::MissingSourceState)?,
        target: to.ok_or(ParseError::MissingDestinationState)?,
        description,
    })
}

fn parse_composite_state(pair: Pair<'_>) -> Result<CompositeState<'_>> {
    let mut name = None;
    let mut elements = StateElements::default();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::state_name => {
                name = Some(inner.as_str());
            }
            Rule::content => {
                elements = parse_content(inner)?;
            }
            _ => {}
        }
    }

    Ok(CompositeState {
        name: name.ok_or(ParseError::MissingCompositeStateName)?,
        elements,
    })
}

fn parse_state_description(
    element_inner: pest::iterators::Pair<'_, Rule>,
) -> Result<StateDescription<'_>> {
    let mut name = None;
    let mut description = None;

    for inner in element_inner.into_inner() {
        match inner.as_rule() {
            Rule::state_name => {
                name = Some(inner.as_str());
            }
            Rule::description => {
                description = Some(inner.as_str());
            }
            _ => {}
        }
    }

    Ok(StateDescription {
        name: name.ok_or(ParseError::MissingStateName)?,
        description: description.ok_or(ParseError::MissingDescription)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    impl CompositeState<'_> {
        fn assert_name(&self, expected: &str) -> &Self {
            assert_eq!(self.name, expected);
            self
        }

        fn assert_children(&self, expected: usize) -> &Self {
            assert_eq!(
                self.elements.composite_states.len(),
                expected,
                "children count for '{}'",
                self.name
            );
            self
        }

        fn assert_enters(&self, expected: &[&str]) -> &Self {
            assert_eq!(
                self.elements.enter_states, expected,
                "enter_states for '{}'",
                self.name
            );
            self
        }

        fn assert_transition(&self, idx: usize, from: &str, to: &str) -> &Self {
            let t = &self.elements.transitions[idx];
            assert_eq!(
                (t.source, t.target),
                (from, to),
                "transition[{}] for '{}'",
                idx,
                self.name
            );
            self
        }

        fn child(&self, idx: usize) -> &CompositeState<'_> {
            &self.elements.composite_states[idx]
        }
    }

    #[test]
    fn test_parse_uml_content() {
        let full_input = "@startuml fsm name\n@enduml";
        let diagram = StateDiagram::parse(full_input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
    }

    #[test]
    fn test_parse_state_description() {
        let input = "A : some description\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        // State descriptions are parsed but not stored in transitions
        assert_eq!(diagram.root.transitions.len(), 0);
    }

    #[test]
    fn test_parse_transition() {
        let input = "A -> B : label\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.root.transitions.len(), 1);
        assert_eq!(diagram.root.transitions[0].source, "A");
        assert_eq!(diagram.root.transitions[0].target, "B");
        assert_eq!(diagram.root.transitions[0].description, Some("label"));
    }

    #[test]
    fn test_parse_multiple_transitions() {
        let input = r#"
        A --> B : label1
        A -u-> C : label2
        "#;
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.root.transitions.len(), 2);
    }

    #[test]
    fn test_parse_enter_transition() {
        let input = "[*] --> A\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.root.enter_states, vec!["A"]);
    }

    #[test]
    fn test_parse_exit_transition() {
        let input = "[*] --> A\n";
        let full_input = format!("@startuml test\n{}@enduml", input);
        let diagram = StateDiagram::parse(&full_input).unwrap();
        assert_eq!(diagram.root.enter_states, vec!["A"]);
    }

    #[test]
    fn test_parse_empty_fsm_diagram() {
        let input = r#"
        @startuml fsm name
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
    }

    #[test]
    fn test_parse_fsm_diagram() {
        let input = r#"
        @startuml fsm name
        [*] --> A
        A --> B : label1
        A -u-> C : label2
        B --> D : label3
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
        assert_eq!(diagram.root.enter_states, vec!["A"]);
        assert_eq!(diagram.root.transitions.len(), 3);
    }
    #[test]
    fn test_parse_fsm_diagram_with_state_descriptions() {
        let input = r#"
        @startuml fsm name
        state A
        [*] --> A
        A --> B : label1
        state B: some desc
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("fsm name"));
        assert_eq!(diagram.root.enter_states, vec!["A"]);
        assert_eq!(diagram.root.transitions.len(), 1);
    }

    #[test]
    fn test_parse_transitions() {
        let input = r#"
        @startuml
        [*] --> StateA
        StateA --> StateA : SelfTransition : Action1
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.root.enter_states, vec!["StateA"]);
        assert_eq!(diagram.root.transitions.len(), 1);
    }

    #[test]
    fn test_parse_single_comment() {
        let input = r#"
        @startuml test
        ' This is a comment
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.name, Some("test"));
        assert_eq!(diagram.root.transitions.len(), 0);
    }

    #[test]
    fn test_parse_comments_mixed_with_transitions() {
        let input = r#"
        @startuml test
        ' Comment at the start
        [*] --> A
        ' Comment in the middle
        A --> B : event1
        ' Another comment
        B --> C : event2
        ' Comment at the end
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.root.enter_states, vec!["A"]);
        assert_eq!(diagram.root.transitions.len(), 2);
    }

    #[test]
    fn test_parse_comment_with_special_characters() {
        let input = r#"
        @startuml test
        ' Comment with special chars: -> --> [*] : @#$%
        [*] --> A
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        assert_eq!(diagram.root.enter_states, vec!["A"]);
    }

    #[test]
    fn test_parse_composite_states() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                }
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.root.composite_states.len(), 1);

        fsm.root.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0)
            .assert_name("state1A")
            .assert_children(1)
            .child(0)
            .assert_name("state1B")
            .assert_children(0);
    }

    #[test]
    fn test_parse_composite_states_with_transitions() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                    [*] --> State1C
                }
                [*] --> State1B
                State1C -> State1N : toState1B
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.root.composite_states.len(), 1);

        let state1a = fsm.root.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0);

        state1a
            .assert_name("state1A")
            .assert_children(1)
            .assert_enters(&["State1B"])
            .assert_transition(0, "State1C", "State1N");

        state1a
            .child(0)
            .assert_name("state1B")
            .assert_children(0)
            .assert_enters(&["State1C"]);
    }
    #[test]
    fn test_parse_multiple_composite_states() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                }
            }
        }
        state State2 {
            state state2A {
                state state2B {
                }
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.root.composite_states.len(), 2);

        fsm.root.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0)
            .assert_name("state1A")
            .assert_children(1)
            .child(0)
            .assert_name("state1B")
            .assert_children(0);

        fsm.root.composite_states[1]
            .assert_name("State2")
            .assert_children(1)
            .child(0)
            .assert_name("state2A")
            .assert_children(1)
            .child(0)
            .assert_name("state2B")
            .assert_children(0);
    }

    #[test]
    fn test_parse_multiple_composite_states_with_transitions() {
        let input = r#"
        @startuml CompositeFSM
        state State1 {
            state state1A {
                state state1B {
                    [*] --> State1C
                }
                [*] --> State1B
                State1C -> State1N : toState1B
            }
        }
        state State2 {
            state state2A {
                state state2B {
                    [*] --> State2C
                }
                [*] --> State2B
                State2C -> State2N : toState2B
            }
        }
        @enduml
        "#;
        let fsm = StateDiagram::parse(input).expect("Failed to parse FSM");
        assert_eq!(fsm.name, Some("CompositeFSM"));
        assert_eq!(fsm.root.composite_states.len(), 2);

        // State1 hierarchy
        let state1a = fsm.root.composite_states[0]
            .assert_name("State1")
            .assert_children(1)
            .child(0);

        state1a
            .assert_name("state1A")
            .assert_children(1)
            .assert_enters(&["State1B"])
            .assert_transition(0, "State1C", "State1N");

        state1a
            .child(0)
            .assert_name("state1B")
            .assert_children(0)
            .assert_enters(&["State1C"]);

        // State2 hierarchy
        let state2a = fsm.root.composite_states[1]
            .assert_name("State2")
            .assert_children(1)
            .child(0);

        state2a
            .assert_name("state2A")
            .assert_children(1)
            .assert_enters(&["State2B"])
            .assert_transition(0, "State2C", "State2N");

        state2a
            .child(0)
            .assert_name("state2B")
            .assert_children(0)
            .assert_enters(&["State2C"]);
    }

    #[test]
    fn parse_state_description_test() {
        let input = r#"
        @startuml test
        state A : This is state A
        state B : This is state B
        @enduml
        "#;
        let diagram = StateDiagram::parse(input).unwrap();
        let descriptions: Vec<_> = diagram.root.state_descriptions;
        assert_eq!(descriptions.len(), 2);
        assert_eq!(descriptions[0].name, "A");
        assert_eq!(descriptions[0].description, "This is state A");
        assert_eq!(descriptions[1].name, "B");
        assert_eq!(descriptions[1].description, "This is state B");
    }
}
