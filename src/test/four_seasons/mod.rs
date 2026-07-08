use crate::{
    error::Result,
    fsm::{
        Action, Event, StateType, TransitionParameters, TransitionTarget, UmlFsm, UmlFsmBuilder,
    },
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_four_seasons_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("PlantFsm");

    // Root level states
    let winter = builder.add_state("Winter", StateType::Enter);
    builder.add_enter_action("Winter", Action::from("WinterIsComing"));
    let spring = builder.add_state("Spring", StateType::Simple);
    let summer = builder.add_state("Summer", StateType::Simple);
    let autumn = builder.add_state("Autumn", StateType::Simple);

    // Root level transitions
    builder.add_transition(TransitionParameters {
        source: "Winter",
        target: TransitionTarget::State("Spring"),
        event: Some(Event("TimeAdvances".into())),
        action: None,
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Spring",
        target: TransitionTarget::State("Summer"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("StartBlooming".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Summer",
        target: TransitionTarget::State("Autumn"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("RipenFruit".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Autumn",
        target: TransitionTarget::State("Winter"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("DropPetals".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });

    // Winter substates
    builder.set_scope(Some(winter));
    builder.add_state("Freezing", StateType::Enter);
    builder.add_state("Mild", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Freezing",
        target: TransitionTarget::State("Mild"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Mild",
        target: TransitionTarget::State("Freezing"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Freezing",
        target: TransitionTarget::State("ArcticBlast"),
        event: None,
        action: Some(Action("StartBlizzard".into())),
        guard: Some(Action("HasVeryColdWeather".into())),
    });
    builder.add_deferred_event("ArcticBlast", Event::from("TemperatureRises"));

    // Spring substates
    builder.set_scope(Some(spring));
    builder.add_state("Brisk", StateType::Enter);
    builder.add_state("Temperate", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Brisk",
        target: TransitionTarget::State("Temperate"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Temperate",
        target: TransitionTarget::State("Brisk"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });

    // Summer substates
    builder.set_scope(Some(summer));
    builder.add_state("Balmy", StateType::Enter);
    builder.add_state("Scorching", StateType::Simple);
    builder.add_enter_action("Scorching", Action::from("StartHeatWave"));
    builder.add_exit_action("Scorching", Action::from("EndHeatWave"));
    builder.add_transition(TransitionParameters {
        source: "Scorching",
        target: TransitionTarget::Internal,
        event: Some(Event("TemperatureRises".into())),
        action: Some(Action("SpontaneousCombustion".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Balmy",
        target: TransitionTarget::State("Scorching"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Scorching",
        target: TransitionTarget::State("Balmy"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });

    // Autumn substates
    builder.set_scope(Some(autumn));
    builder.add_state("Crisp", StateType::Enter);
    builder.add_state("Pleasant", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Crisp",
        target: TransitionTarget::State("Pleasant"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Pleasant",
        target: TransitionTarget::State("Crisp"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn four_seasons() -> Self {
        let path = get_adjacent_file_path(file!(), "four_seasons.puml");
        Self {
            name: "four_seasons",
            content: include_str!("./four_seasons.puml"),
            parsed: build_four_seasons_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
