# Phyto FSM

A Rust procedural macro for generating state machines from PlantUML diagrams.

[![CI](https://github.com/TobTheRock/phytofsm/actions/workflows/ci.yml/badge.svg)](https://github.com/TobTheRock/phytofsm/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/phyto-fsm)](https://crates.io/crates/phyto-fsm)
[![Documentation](https://docs.rs/phyto-fsm/badge.svg)](https://docs.rs/phyto-fsm)

## Overview

Many state machine frameworks provide the possibility to create a visualization from your FSM
defined by the source code, which then can be e.g. added to your documentation.
`phyto-fsm` does it the opposite way:

- you create a visual representation of an FSM using a [PlantUML state diagram](https://plantuml.com/state-diagram)
- `phyto-fsm`
  - parses the diagram during compile time
  - generates a type-safe FSM representation as well as a trait to handle actions such as on transitions
- using the generated trait you define the actions to take on transitions

This way the design of the FSM is easy to grasp first hand and documentation and source code are always inline.

## State Machine Features

| Feature | Description | Example |
|---------|-------------|---------|
| Events with custom data | Trigger transitions with typed event parameters | [actions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/actions.rs) [data_types.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/data_types.rs) |
| Actions on transitions | Execute custom code when transitions occur | [actions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/actions.rs) |
| Enter/exit actions | Execute custom code when entering or exiting a state | [enter_exit.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/enter_exit.rs) |
| Composite states | Nested/hierarchical states with automatic enter state resolution | [composite_states.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/composite_states.rs) |
| Substate-to-substate transitions | Transitions between substates across different parent states | [substate_to_substate.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/substate_to_substate.rs) |
| Self-transitions | States that transition to themselves | [transitions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/transitions.rs) |
| Alternative transitions | Multiple transitions from the same state with different events | [transitions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/transitions.rs) |
| Event lists | One transition triggered by any of several comma-separated events | [transitions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/transitions.rs) |
| Guard conditions | Conditional transitions using `[GuardName]` syntax | [guards.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/guards.rs) |
| Internal transitions | Stay in state without triggering exit/enter actions | [internal_transitions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/internal_transitions.rs) |
| Direct transitions | Automatic transitions without events, with optional guards and actions | [direct_transitions.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/direct_transitions.rs) |
| Deferred events | Events deferred in one state are re-evaluated after transitioning to another state | [deferred_events.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/deferred_events.rs) |
| Transition logging | Optional runtime logging via [log](https://docs.rs/log/latest/log/) crate | [four_seasons.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/four_seasons/main.rs) |
| Custom naming templates | Override default naming patterns for generated types | [custom_naming.rs](https://github.com/TobTheRock/phytofsm/blob/main/tests/custom_naming.rs) |

### Missing Features

- Pseudo States
  - exit
  - history states
  - ...
- sub state machines
- orthogonal regions

## UML Syntax for FSM Actions & Events

In order to generate a state machine this library requires transitions and actions to be described
according to the [UML specification](https://www.omg.org/spec/UML/2.5.1/PDF).

### Transitions

A transition moves the FSM from one state to another when an event fires:

```puml
StateA --> StateB : EventName
```

- **EventName**: The event that triggers the transition

#### Event Lists

A single transition can be triggered by any of several events by listing them comma-separated:

```puml
StateA --> StateB : EventA, EventB, EventC
```

This desugars to one transition per event — equivalent to writing three separate `StateA --> StateB` lines. Because each event carries its own parameter type (and actions/guards are unique per event), an event list cannot share an action or guard across its events; use it for plain "any of these events transitions here" cases.

#### Transitions with Actions

Transitions can optionally include an action that is executed when the transition occurs:

```puml
StateA --> StateB : EventName / ActionName
```

- **ActionName** (optional): The action method called during the transition, separated by `/`

**Be aware that actions MUST be unique per event**, else a compile time error is raised.

#### Run-to-Completion (RTC)

Transition actions are atomic. An action cannot trigger another event on the same FSM — Rust's enforces this with its borrow sematincs, the FSM is mutably borrowed during the entire transition.
If you need actions to trigger follow-up events, use an external event loop (see [rtc example](https://github.com/TobTheRock/phytofsm/blob/main/examples/rtc.rs)).

### Guards

Transitions can include a guard condition that must evaluate to `true` for the transition to occur:

```puml
StateA --> StateB : EventName [GuardName] / ActionName
```

- **GuardName** (optional): A boolean method called before the transition. If it returns `false`, the transition is skipped.

When multiple transitions share the same event, guards are evaluated in declaration order and the first matching guard wins:

```puml
StateA --> StateB : Change [IsReady] / GoToB
StateA --> StateC : Change [IsFull] / GoToC
```

**Guards MUST be unique per event** — using the same guard name on multiple transitions for the same event will produce a compile-time error.

The generated actions trait will include guard methods with the signature:

```rust,ignore
fn guard_name(&self, event: &Self::EventNameParams) -> bool;
```

### Internal Transitions

Internal transitions execute an action without leaving the current state. Unlike self-transitions, they do **not** trigger exit or enter actions:

```puml
StateA : EventName / ActionName
```

This is equivalent to a transition with no target state. When `EventName` fires while in `StateA`, `ActionName` is called but the state remains unchanged and no exit/enter actions run.

Compare with a **self-transition**, which does trigger exit and re-entry:

```puml
StateA -> StateA : EventName / ActionName
```

Internal transitions also work inside composite states — they will not trigger the parent state's exit/enter actions.

### Direct Transitions

Direct transitions fire automatically without an event. They are evaluated on FSM start and after every event-based transition:

```puml
StateA --> StateB : / ActionName
StateA --> StateC : [GuardName] / ActionName
StateA --> StateD : [GuardName]
```

- **No event name**: The absence of an event name makes this a direct transition
- **GuardName** (optional): A boolean method called to decide if the transition should fire
- **ActionName** (optional): An action executed when the transition fires

When multiple guarded direct transitions exist on a state, they are evaluated in declaration order and the first matching guard wins. If no guard matches, the state remains unchanged.

Direct transition actions and guards have no event parameters:

```rust,ignore
fn action_name(&mut self);
fn guard_name(&self) -> bool;
```

### Deferred Events

Events can be deferred in a state using the `/defer` syntax. A deferred event is stored and re-evaluated whenever the FSM transitions to a new state:

```puml
StateA : EventName /defer
```

When `EventName` occurs while in `StateA`, instead of being discarded it is placed in a deferred event queue. After any subsequent state transition, all deferred events are re-evaluated in the new state:

- If the new state also defers the event, it stays in the queue (re-deferred)
- If the new state handles the event, a transition fires
- If the new state neither defers nor handles the event, it is silently discarded

```puml
StateA : GoToA /defer
StateA -> StateB : GoToB
StateB -> StateA : GoToA
```

In this example, sending `GoToA` while in `StateA` defers it. Sending `GoToB` transitions to `StateB`, where the deferred `GoToA` fires and transitions back to `StateA`.

The deferred event queue and associated `defer_event` function pointer are only generated when at least one state in the FSM uses `/defer`.

**Note on event parameter lifetimes:** Deferred events are stored in an internal queue until they can be processed, which means event parameters must live long enough to outlast the deferral. If your event parameter types contain references (e.g. `&str`, `&[u8]`), the borrowed data must remain valid until the deferred event is eventually consumed or discarded. In practice, prefer owned types (e.g. `String`, `Vec<u8>`) for event parameters when deferral is involved.

### Enter/Exit Actions

States can define enter and exit actions using the UML state description syntax:

```puml
State MyState : entry / EnterActionName
State MyState : exit / ExitActionName
```

- **`entry /`** prefix denotes an **enter action** - called when entering the state
- **`exit /`** prefix denotes an **exit action** - called when leaving the state

Enter and exit actions can also be defined within composite states:

```puml
state Parent {
  Parent: entry / EnterParent
  Parent: exit / ExitParent

  state Child : entry / EnterChild
  state Child : exit / ExitChild

  [*] --> Child
}
```

**Execution order:**

Following the [UML specification](https://www.omg.org/spec/UML/2.5.1/PDF), exit actions are always executed before enter actions during a transition:

1. **Exit** the source state (and its parents, from innermost to outermost)
2. **Enter** the target state (and its parents, from outermost to innermost)

Furthermore:

- Transitions between substates of the same parent do not trigger the parent's enter/exit actions
- Internal transitions (no target) do not trigger any exit/enter actions
- Self-transitions trigger both exit and enter actions (exit first, then enter)

## Generated Code

When you use `generate_fsm!("path/to/diagram.puml")`, the macro generates various traits, enums, and structs based on your PlantUML diagram name and elements. Here's how they are named:

### Core Generated Types

For a PlantUML diagram whose name is given by:

```text
@startuml DiagramName
...
@enduml
```

The following is generated:

| Generated Item | Naming Pattern | Description |
|---------------|----------------|-------------|
| **FSM Struct** | `{DiagramName}` | Main state machine struct (UpperCamelCase) |
| **Event Parameters Trait** | `I{DiagramName}EventParams` | Trait defining event parameter types |
| **Actions Trait** | `I{DiagramName}Actions` | Trait defining action methods |
| **State ID Enum** | `{DiagramName}State` | Enum returned by `current_state()` |
| **Module** | `{diagram_name}` | Generated module name (snake_case) |

### Events and Actions

From the elements given by the custom syntax the following is derived:

| PlantUML Element | Generated Item | Naming Pattern |
|-----------------|----------------|----------------|
| **Event** | Method name of the FSM Struct | snake_case of event name |
| **Event** | Parameter type | `{EventName}Params` |
| **Action** (Transition/Enter/Exit) | Method name of the Actions Trait | snake_case of action name |
| **State** | State name | Preserved as written in PlantUML |

### Custom Naming Templates

The default naming patterns for the publicly exposed generated types can be customized by providing a naming template file via the `naming` option.
The template file uses `{name}` (UpperCamelCase) as placeholder. The `module` value is automatically converted to snake_case.
See the [default template](src/codegen/default_naming.tmpl) for the format and [custom_naming](tests/custom_naming.rs) for a usage example.

Note: Event, action, and state element names are not affected by the naming template — they always use the names as written in the PlantUML diagram.

## Debugging the Proc Macro

To see debug output from the proc macro at compile time on stderr (e.g. parsed diagrams, FSM builder steps), set the `PHYTO_DEBUG` environment variable:

```sh
PHYTO_DEBUG=1 cargo build
```

## Example

### 1. Create a PlantUML state diagram

```puml
@startuml PlantFsm

state Winter {
  Winter: entry / WinterIsComing

  state Freezing
  state Mild

  [*] --> Freezing
  Freezing -> Mild: TemperatureRises
  Mild -> Freezing: TemperatureDrops
}

state Spring {
  state Chilly
  state Warm
  [*] --> Chilly
  Chilly -> Warm: TemperatureRises
  Warm -> Chilly: TemperatureDrops
}

state Summer {
  state Balmy
  state Scorching: entry / StartHeatWave
  state Scorching: exit / EndHeatWave
  state Scorching: TemperatureRises / SpontaneousCombustion

  [*] --> Balmy
  Balmy -> Scorching: TemperatureRises
  Scorching -> Balmy: TemperatureDrops
}

[*] --> Winter
Winter --> Spring : TimeAdvances [EnoughTimePassed]
Spring --> Summer : TimeAdvances [EnoughTimePassed] / StartBlooming
Summer --> Autumn : TimeAdvances [EnoughTimePassed] / RipenFruit
Autumn --> Winter : TimeAdvances [EnoughTimePassed] / DropPetals

@enduml
```

### 2. Generate the state machine

```rust,ignore
use phyto_fsm::generate_fsm;

// Generate FSM from PlantUML file
generate_fsm!(
    file_path = "path/to/your/diagram.puml",
    log_level = "debug"  // Optional: enables transition logging
);
```

### 3. Implement your actions

```rust,ignore
use plant_fsm::{IPlantFsmActions, IPlantFsmEventParams, NoEventData};

struct PlantActions;

impl IPlantFsmEventParams for PlantActions {
    type TimeAdvancesParams = std::time::SystemTime;
    type TemperatureRisesParams = NoEventData;
    type TemperatureDropsParams = NoEventData;
}

impl IPlantFsmActions for PlantActions {
    // Transition actions
    fn start_blooming(&mut self, time: Self::TimeAdvancesParams) {
        println!("Started blooming at {:?}", time);
    }

    fn ripen_fruit(&mut self, _: Self::TimeAdvancesParams) {
        println!("Fruit is ripening");
    }

    fn drop_petals(&mut self, _: Self::TimeAdvancesParams) {
        println!("Dropping petals");
    }

    // Internal transition action (no exit/enter triggered)
    fn spontaneous_combustion(&mut self, _: Self::TemperatureRisesParams) {
        println!("Too hot! Spontaneous combustion!");
    }

    // Enter actions
    fn winter_is_coming(&mut self) {
        println!("Brace yourselves, winter is coming!");
    }

    fn start_heat_wave(&mut self) {
        println!("Heat wave starting!");
    }

    // Exit actions
    fn end_heat_wave(&mut self) {
        println!("Heat wave ending, cooling down...");
    }

    // Guards
    fn enough_time_passed(&self, _: &Self::TimeAdvancesParams) -> bool {
        true // your logic here
    }
}
```

### 4. Use your state machine

```rust,ignore
fn main() {
    let actions = PlantActions;
    // Creating the FSM triggers winter_is_coming() enter action
    let mut fsm = plant_fsm::start(actions);

    // Transition within Winter: Freezing -> Mild
    fsm.temperature_rises(());

    // Transition to next season: Winter -> Spring (enters Spring::Chilly)
    fsm.time_advances(std::time::SystemTime::now());

    // Transition to Summer, then enter Scorching (triggers start_heat_wave)
    fsm.time_advances(std::time::SystemTime::now());
    fsm.temperature_rises(());

    // Leave Scorching (triggers end_heat_wave), go to Balmy
    fsm.temperature_drops(());
}
```
