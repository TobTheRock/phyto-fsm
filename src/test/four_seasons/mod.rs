use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_four_seasons_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("PlantFsm");

    // Root level states
    let winter = builder.add_enter_state("Winter");
    builder.add_enter_action("Winter", Action::from("WinterIsComing"));
    let spring = builder.add_state("Spring");
    let summer = builder.add_state("Summer");
    let autumn = builder.add_state("Autumn");

    // Root level transitions
    builder.add_transition(TransitionParameters {
        source: "Winter",
        target: Some("Spring"),
        event: Some(Event("TimeAdvances".into())),
        action: None,
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Spring",
        target: Some("Summer"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("StartBlooming".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Summer",
        target: Some("Autumn"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("RipenFruit".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters {
        source: "Autumn",
        target: Some("Winter"),
        event: Some(Event("TimeAdvances".into())),
        action: Some(Action("DropPetals".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });

    // Winter substates
    builder.set_scope(Some(winter));
    builder.add_enter_state("Freezing");
    builder.add_state("Mild");
    builder.add_transition(TransitionParameters {
        source: "Freezing",
        target: Some("Mild"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Mild",
        target: Some("Freezing"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Freezing",
        target: Some("ArcticBlast"),
        event: None,
        action: Some(Action("StartBlizzard".into())),
        guard: Some(Action("HasVeryColdWeather".into())),
    });
    builder.add_deferred_event("ArcticBlast", Event::from("TemperatureRises"));

    // Spring substates
    builder.set_scope(Some(spring));
    builder.add_enter_state("Brisk");
    builder.add_state("Temperate");
    builder.add_transition(TransitionParameters {
        source: "Brisk",
        target: Some("Temperate"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Temperate",
        target: Some("Brisk"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });

    // Summer substates
    builder.set_scope(Some(summer));
    builder.add_enter_state("Balmy");
    builder.add_state("Scorching");
    builder.add_enter_action("Scorching", Action::from("StartHeatWave"));
    builder.add_exit_action("Scorching", Action::from("EndHeatWave"));
    builder.add_transition(TransitionParameters {
        source: "Scorching",
        target: None,
        event: Some(Event("TemperatureRises".into())),
        action: Some(Action("SpontaneousCombustion".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Balmy",
        target: Some("Scorching"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Scorching",
        target: Some("Balmy"),
        event: Some(Event("TemperatureDrops".into())),
        action: None,
        guard: None,
    });

    // Autumn substates
    builder.set_scope(Some(autumn));
    builder.add_enter_state("Crisp");
    builder.add_state("Pleasant");
    builder.add_transition(TransitionParameters {
        source: "Crisp",
        target: Some("Pleasant"),
        event: Some(Event("TemperatureRises".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Pleasant",
        target: Some("Crisp"),
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
