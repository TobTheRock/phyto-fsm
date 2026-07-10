use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_four_seasons_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("PlantFsm");

    // Root level states
    let winter = builder.add_transition(TransitionParameters::Enter { target: "Winter" });
    builder.add_enter_action("Winter", Action::from("WinterIsComing"));
    let spring = builder.add_state("Spring");
    let summer = builder.add_state("Summer");
    let autumn = builder.add_state("Autumn");

    // Root level transitions
    builder.add_transition(TransitionParameters::Event {
        source: "Winter",
        target: "Spring",
        event: Event("TimeAdvances".into()),
        action: None,
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Spring",
        target: "Summer",
        event: Event("TimeAdvances".into()),
        action: Some(Action("StartBlooming".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Summer",
        target: "Autumn",
        event: Event("TimeAdvances".into()),
        action: Some(Action("RipenFruit".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Autumn",
        target: "Winter",
        event: Event("TimeAdvances".into()),
        action: Some(Action("DropPetals".into())),
        guard: Some(Action("EnoughTimePassed".into())),
    });

    // Winter substates
    builder.set_scope(Some(winter));
    builder.add_transition(TransitionParameters::Enter { target: "Freezing" });
    builder.add_state("Mild");
    builder.add_transition(TransitionParameters::Event {
        source: "Freezing",
        target: "Mild",
        event: Event("TemperatureRises".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Mild",
        target: "Freezing",
        event: Event("TemperatureDrops".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Direct {
        source: "Freezing",
        target: "ArcticBlast",
        action: Some(Action("StartBlizzard".into())),
        guard: Some(Action("HasVeryColdWeather".into())),
    });
    builder.add_deferred_event("ArcticBlast", Event::from("TemperatureRises"));

    // Spring substates
    builder.set_scope(Some(spring));
    builder.add_transition(TransitionParameters::Enter { target: "Brisk" });
    builder.add_state("Temperate");
    builder.add_transition(TransitionParameters::Event {
        source: "Brisk",
        target: "Temperate",
        event: Event("TemperatureRises".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Temperate",
        target: "Brisk",
        event: Event("TemperatureDrops".into()),
        action: None,
        guard: None,
    });

    // Summer substates
    builder.set_scope(Some(summer));
    builder.add_transition(TransitionParameters::Enter { target: "Balmy" });
    builder.add_state("Scorching");
    builder.add_enter_action("Scorching", Action::from("StartHeatWave"));
    builder.add_exit_action("Scorching", Action::from("EndHeatWave"));
    builder.add_transition(TransitionParameters::Internal {
        source: "Scorching",
        event: Event("TemperatureRises".into()),
        action: Some(Action("SpontaneousCombustion".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Balmy",
        target: "Scorching",
        event: Event("TemperatureRises".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Scorching",
        target: "Balmy",
        event: Event("TemperatureDrops".into()),
        action: None,
        guard: None,
    });

    // Autumn substates
    builder.set_scope(Some(autumn));
    builder.add_transition(TransitionParameters::Enter { target: "Crisp" });
    builder.add_state("Pleasant");
    builder.add_transition(TransitionParameters::Event {
        source: "Crisp",
        target: "Pleasant",
        event: Event("TemperatureRises".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Pleasant",
        target: "Crisp",
        event: Event("TemperatureDrops".into()),
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
