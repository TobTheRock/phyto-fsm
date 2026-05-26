use log::debug;

pub trait IPlantFsmEventParams {
    type TemperatureRisesParams;
    type TemperatureDropsParams;
    type TimeAdvancesParams;
}

pub trait IPlantFsmActions: IPlantFsmEventParams {
    // transition actions
    fn start_blooming(&mut self, event: Self::TimeAdvancesParams);
    fn ripen_fruit(&mut self, event: Self::TimeAdvancesParams);
    fn drop_petals(&mut self, event: Self::TimeAdvancesParams);
    fn spontaneous_combustion(&mut self, event: Self::TemperatureRisesParams);
    // direct transition actions
    fn start_blizzard(&mut self);
    // enter actions
    fn start_heat_wave(&mut self);
    fn winter_is_coming(&mut self);
    // exit actions
    fn end_heat_wave(&mut self);
    // guards
    fn enough_time_passed(&self, event: &Self::TimeAdvancesParams) -> bool;
    fn has_very_cold_weather(&self) -> bool;
}

type NoEventData = ();

enum PlantFsmEvent<T: IPlantFsmActions> {
    TemperatureRises(T::TemperatureRisesParams),
    TemperatureDrops(T::TemperatureDropsParams),
    TimeAdvances(T::TimeAdvancesParams),
}

impl<A> std::fmt::Display for PlantFsmEvent<A>
where
    A: IPlantFsmActions,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PlantFsmEvent::TemperatureRises(_) => "TemperatureRises",
            PlantFsmEvent::TemperatureDrops(_) => "TemperatureDrops",
            PlantFsmEvent::TimeAdvances(_) => "TimeAdvances",
        };

        write!(f, "{}", name)
    }
}

#[derive(Copy)]
struct PlantFsmStateImpl<T: IPlantFsmActions> {
    id: PlantFsmState,
    // Transition based on an event, depending on the state
    // Returns Some(state) if consumed, None if not consumed
    transition: fn(event: PlantFsmEvent<T>, actions: &mut T) -> Option<Self>,
    // Direct transitions, not based on an event
    direct_transition: fn(actions: &mut T) -> Option<Self>,
    // state to enter when transitioned to, if there are no substates this is Self
    enter_state: fn() -> Self,
    // enter action, composite states check for internal transitions
    enter: fn(actions: &mut T, from: &Self) -> (),
    // exit action, composite states check for internal transitions
    exit: fn(actions: &mut T, to: &Self) -> (),
    // check if event would be deferred, only generated if the FSM contains deferred events
    defer_event: fn(event: &PlantFsmEvent<T>) -> bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PlantFsmState {
    Winter,
    WinterMild,
    WinterFreezing,
    WinterArcticBlast,
    Spring,
    SpringBrisk,
    SpringTemperate,
    Summer,
    SummerBalmy,
    SummerScorching,
    Autumn,
    AutumnCrisp,
    AutumnPleasant,
    _PlantFsmInitialState_,
}

impl From<PlantFsmState> for &'static str {
    fn from(name: PlantFsmState) -> Self {
        match name {
            PlantFsmState::Winter => "Winter",
            PlantFsmState::WinterArcticBlast => "Winter::ArcticBlast",
            PlantFsmState::WinterMild => "Winter::Mild",
            PlantFsmState::WinterFreezing => "Winter::Freezing",
            PlantFsmState::Spring => "Spring",
            PlantFsmState::SpringBrisk => "Spring::Brisk",
            PlantFsmState::SpringTemperate => "Spring::Temperate",
            PlantFsmState::Summer => "Summer",
            PlantFsmState::SummerBalmy => "Summer::Balmy",
            PlantFsmState::SummerScorching => "Summer::Scorching",
            PlantFsmState::Autumn => "Autumn",
            PlantFsmState::AutumnCrisp => "Autumn::Crisp",
            PlantFsmState::AutumnPleasant => "Autumn::Pleasant",
            PlantFsmState::_PlantFsmInitialState_ => "[*]",
        }
    }
}

impl std::fmt::Display for PlantFsmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = (*self).into();
        write!(f, "{}", name)
    }
}

impl<T: IPlantFsmActions> Clone for PlantFsmStateImpl<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            transition: self.transition,
            direct_transition: self.direct_transition,
            enter_state: self.enter_state,
            enter: self.enter,
            exit: self.exit,
            defer_event: self.defer_event,
        }
    }
}

impl<T: IPlantFsmActions> PartialEq for PlantFsmStateImpl<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> PlantFsmStateImpl<T>
where
    T: IPlantFsmActions,
{
    // Hidden init state to trigger enter actions when starting the FSM
    fn init() -> Self {
        Self {
            id: PlantFsmState::_PlantFsmInitialState_,
            transition: |_event, _action| None,
            direct_transition: |_action| Some(PlantFsmStateImpl::winter_freezing()),
            enter_state: Self::init,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
            defer_event: |_event| false,
        }
    }

    fn winter() -> Self {
        Self {
            id: PlantFsmState::Winter,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    Some(Self::spring())
                }
                _ => None,
            },
            direct_transition: |_action| None,
            enter_state: Self::winter_freezing,
            enter: |actions, from| {
                if matches!(
                    from.id,
                    PlantFsmState::WinterFreezing | PlantFsmState::WinterMild
                ) {
                    return;
                }
                actions.winter_is_coming();
            },
            exit: |_actions, _to| {},
            defer_event: |_event| false,
        }
    }

    fn winter_freezing() -> Self {
        Self {
            id: PlantFsmState::WinterFreezing,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::winter_mild()),
                // Check the parent
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |action| {
                if action.has_very_cold_weather() {
                    action.start_blizzard();
                    Some(Self::winter_arctic_blast())
                } else {
                    None
                }
            },
            enter_state: Self::winter_freezing,
            enter: |actions, from| (Self::winter().enter)(actions, from),
            exit: |actions, to| (Self::winter().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn winter_mild() -> Self {
        Self {
            id: PlantFsmState::WinterMild,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::winter_freezing()),
                _ => {
                    let parent = Self::winter();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::winter_mild,
            enter: |actions, from| (Self::winter().enter)(actions, from),
            exit: |actions, to| (Self::winter().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn winter_arctic_blast() -> Self {
        Self {
            id: PlantFsmState::WinterArcticBlast,
            transition: |event, actions| {
                let parent = Self::winter();
                (parent.transition)(event, actions)
            },
            direct_transition: |_action| None,
            enter_state: Self::winter_arctic_blast,
            enter: |actions, from| (Self::winter().enter)(actions, from),
            exit: |actions, to| (Self::winter().exit)(actions, to),
            defer_event: |event| matches!(event, PlantFsmEvent::TemperatureRises(_)),
        }
    }

    fn spring() -> Self {
        Self {
            id: PlantFsmState::Spring,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.start_blooming(params);
                    Some(Self::summer())
                }
                _ => None,
            },
            direct_transition: |_action| None,
            enter_state: Self::spring_brisk,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
            defer_event: |_event| false,
        }
    }

    fn spring_brisk() -> Self {
        Self {
            id: PlantFsmState::SpringBrisk,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::spring_temperate()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::spring_brisk,
            enter: |actions, from| (Self::spring().enter)(actions, from),
            exit: |actions, to| (Self::spring().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn spring_temperate() -> Self {
        Self {
            id: PlantFsmState::SpringTemperate,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::spring_brisk()),
                _ => {
                    let parent = Self::spring();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::spring_temperate,
            enter: |actions, from| (Self::spring().enter)(actions, from),
            exit: |actions, to| (Self::spring().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn summer() -> Self {
        Self {
            id: PlantFsmState::Summer,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.ripen_fruit(params);
                    Some(Self::autumn())
                }
                _ => None,
            },
            direct_transition: |_action| None,
            enter_state: Self::summer_balmy,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
            defer_event: |_event| false,
        }
    }

    fn summer_balmy() -> Self {
        Self {
            id: PlantFsmState::SummerBalmy,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::summer_scorching()),
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::summer_balmy,
            enter: |actions, from| (Self::summer().enter)(actions, from),
            exit: |actions, to| (Self::summer().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn summer_scorching() -> Self {
        Self {
            id: PlantFsmState::SummerScorching,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::summer_balmy()),
                PlantFsmEvent::TemperatureRises(params) => {
                    actions.spontaneous_combustion(params);
                    Some(Self::summer_scorching())
                }
                _ => {
                    let parent = Self::summer();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::summer_scorching,
            enter: |actions, _from| actions.start_heat_wave(),
            exit: |actions, _to| actions.end_heat_wave(),
            defer_event: |_event| false,
        }
    }

    fn autumn() -> Self {
        Self {
            id: PlantFsmState::Autumn,
            transition: |event, actions| match event {
                PlantFsmEvent::TimeAdvances(params) if actions.enough_time_passed(&params) => {
                    actions.drop_petals(params);
                    Some(Self::winter())
                }
                _ => None,
            },
            direct_transition: |_action| None,
            enter_state: Self::autumn_crisp,
            enter: |_actions, _from| {},
            exit: |_actions, _to| {},
            defer_event: |_event| false,
        }
    }

    fn autumn_crisp() -> Self {
        Self {
            id: PlantFsmState::AutumnCrisp,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureRises(_) => Some(Self::autumn_pleasant()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::autumn_crisp,
            enter: |actions, from| (Self::autumn().enter)(actions, from),
            exit: |actions, to| (Self::autumn().exit)(actions, to),
            defer_event: |_event| false,
        }
    }

    fn autumn_pleasant() -> Self {
        Self {
            id: PlantFsmState::AutumnPleasant,
            transition: |event, actions| match event {
                PlantFsmEvent::TemperatureDrops(_) => Some(Self::autumn_crisp()),
                _ => {
                    let parent = Self::autumn();
                    (parent.transition)(event, actions)
                }
            },
            direct_transition: |_action| None,
            enter_state: Self::autumn_pleasant,
            enter: |actions, from| (Self::autumn().enter)(actions, from),
            exit: |actions, to| (Self::autumn().exit)(actions, to),
            defer_event: |_event| false,
        }
    }
}

struct PlantFsmImpl<A: IPlantFsmActions> {
    actions: A,
    current_state: PlantFsmStateImpl<A>,
    // This member is only generated when the FSM contains deferred events
    deferred_events: std::collections::VecDeque<PlantFsmEvent<A>>,
}

impl<A> PlantFsmImpl<A>
where
    A: IPlantFsmActions,
{
    fn start(actions: A) -> Self {
        let mut fsm = Self {
            actions,
            current_state: PlantFsmStateImpl::init(),
            deferred_events: std::collections::VecDeque::new(),
        };

        fsm.try_direct_transition();
        fsm
    }

    fn run_event_loop(&mut self, event: PlantFsmEvent<A>) {
        let pending = std::mem::take(&mut self.deferred_events);

        std::iter::once(event).chain(pending).for_each(|event| {
            self.process_event(event);
        });
    }

    fn process_event(&mut self, event: PlantFsmEvent<A>) {
        let Some(event) = self.try_defer_event(event) else {
            return;
        };
        if self.try_event_based_transition(event) {
            self.try_direct_transition();
        }
    }

    fn try_defer_event(&mut self, event: PlantFsmEvent<A>) -> Option<PlantFsmEvent<A>> {
        if (self.current_state.defer_event)(&event) {
            debug!(
                "PlantFsm: {} deferring event {}",
                self.current_state.id, event
            );
            self.deferred_events.push_back(event);
            None
        } else {
            Some(event)
        }
    }

    fn try_event_based_transition(&mut self, event: PlantFsmEvent<A>) -> bool {
        let event_name = format!("{}", event);
        if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
            if transition_state.id != self.current_state.id {
                let enter_state = (transition_state.enter_state)();
                debug!(
                    "PlantFsm: {} -[{}]-> {}, entering {}",
                    self.current_state.id, event_name, transition_state.id, enter_state.id
                );
                self.change_state(enter_state);
            }
            true
        } else {
            false
        }
    }

    fn try_direct_transition(&mut self) {
        while let Some(transition_state) = (self.current_state.direct_transition)(&mut self.actions)
        {
            let enter_state = (transition_state.enter_state)();
            debug!(
                "PlantFsm: {} -[direct]-> {}, entering {}",
                self.current_state.id, enter_state.id, transition_state.id
            );
            self.change_state(enter_state);
        }
    }

    fn change_state(&mut self, next_state: PlantFsmStateImpl<A>) {
        (self.current_state.exit)(&mut self.actions, &next_state);
        (next_state.enter)(&mut self.actions, &self.current_state);
        self.current_state = next_state;
    }
}

pub struct PlantFsm<A: IPlantFsmActions>(PlantFsmImpl<A>);
// This may already trigger actions, depending on the initial state
pub fn start<A: IPlantFsmActions>(actions: A) -> PlantFsm<A> {
    PlantFsm(PlantFsmImpl::start(actions))
}

impl<A> PlantFsm<A>
where
    A: IPlantFsmActions,
{
    pub fn current_state(&self) -> PlantFsmState {
        self.0.current_state.id
    }

    pub fn temperature_rises(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureRisesParams,
    ) {
        self.0
            .run_event_loop(PlantFsmEvent::TemperatureRises(params));
    }

    pub fn temperature_drops(
        &mut self,
        params: <A as IPlantFsmEventParams>::TemperatureDropsParams,
    ) {
        self.0
            .run_event_loop(PlantFsmEvent::TemperatureDrops(params));
    }

    pub fn time_advances(&mut self, params: <A as IPlantFsmEventParams>::TimeAdvancesParams) {
        self.0.run_event_loop(PlantFsmEvent::TimeAdvances(params));
    }
}

#[cfg(test)]
mod test {
    use mockall::{mock, predicate};

    use super::{IPlantFsmActions, IPlantFsmEventParams, NoEventData};

    mock! {
        PlantFsmActions {}
        impl IPlantFsmActions for PlantFsmActions {
            fn start_blooming(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn ripen_fruit(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn drop_petals(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams);
            fn spontaneous_combustion(&mut self, event: <MockPlantFsmActions as IPlantFsmEventParams>::TemperatureRisesParams);
            fn start_blizzard(&mut self);

            fn start_heat_wave(&mut self);
            fn winter_is_coming(&mut self);

            fn end_heat_wave(&mut self);

            fn enough_time_passed(
                &self,
                event: &<MockPlantFsmActions as IPlantFsmEventParams>::TimeAdvancesParams,
            ) -> bool;
            fn has_very_cold_weather(&self) -> bool;
        }
    }

    impl IPlantFsmEventParams for MockPlantFsmActions {
        type TemperatureRisesParams = NoEventData;
        type TemperatureDropsParams = NoEventData;
        type TimeAdvancesParams = u32;
    }

    fn setup() -> MockPlantFsmActions {
        let _ = stderrlog::new().verbosity(log::Level::Debug).init();
        MockPlantFsmActions::new()
    }

    #[test]
    fn test_transitions() {
        let time = 42;
        let mut actions = setup();

        actions
            .expect_enough_time_passed()
            .returning(|_| true)
            .times(4);

        actions
            .expect_start_blooming()
            .returning(|_| ())
            .with(predicate::eq(time))
            .times(1);
        actions.expect_ripen_fruit().returning(|_| ()).times(1);
        actions.expect_drop_petals().returning(|_| ()).times(1);
        // Called twice: once on init (entering Winter::Freezing), once when returning from Autumn
        actions.expect_winter_is_coming().returning(|| ()).times(2);
        actions.expect_has_very_cold_weather().returning(|| false);

        let mut fsm = super::start(actions);
        fsm.temperature_rises(());
        fsm.time_advances(time);
        fsm.time_advances(time);
        fsm.temperature_drops(());
        fsm.time_advances(time);
        fsm.time_advances(time);
    }

    #[test]
    fn test_deferred_event_fires_after_state_change() {
        let mut actions = setup();

        // Enter ArcticBlast: cold weather guard triggers direct transition
        actions
            .expect_has_very_cold_weather()
            .returning(|| true)
            .times(1);
        actions.expect_start_blizzard().returning(|| ()).times(1);
        actions.expect_winter_is_coming().returning(|| ()).times(1);

        // TimeAdvances will trigger Winter→Spring (guard passes)
        actions
            .expect_enough_time_passed()
            .returning(|_| true)
            .times(1);

        // No other actions expected
        actions.expect_start_blooming().never();
        actions.expect_ripen_fruit().never();
        actions.expect_drop_petals().never();
        actions.expect_spontaneous_combustion().never();
        actions.expect_start_heat_wave().never();
        actions.expect_end_heat_wave().never();

        let mut fsm = super::start(actions);
        assert_eq!(fsm.current_state(), super::PlantFsmState::WinterArcticBlast);

        // Defer TemperatureRises while in ArcticBlast
        fsm.temperature_rises(());
        assert_eq!(fsm.current_state(), super::PlantFsmState::WinterArcticBlast);
        assert_eq!(
            fsm.0.deferred_events.len(),
            1,
            "Event should be deferred, not lost"
        );

        // TimeAdvances transitions Winter→Spring::Brisk
        // Then deferred TemperatureRises fires: Brisk→Temperate
        fsm.time_advances(42);
        assert_eq!(
            fsm.current_state(),
            super::PlantFsmState::SpringTemperate,
            "Deferred TemperatureRises should have fired in Spring::Brisk"
        );
        assert_eq!(
            fsm.0.deferred_events.len(),
            0,
            "Deferred event should be consumed"
        );
    }

    #[test]
    fn test_substate_enter_exit_actions() {
        let time = 42;
        let mut actions = setup();
        actions
            .expect_enough_time_passed()
            .returning(|_| true)
            .times(2);

        // Called once on init (entering Winter::Freezing)
        actions.expect_winter_is_coming().returning(|| ()).times(1);
        actions.expect_has_very_cold_weather().returning(|| false);
        actions.expect_start_blooming().returning(|_| ()).times(1);
        actions.expect_start_heat_wave().returning(|| ()).times(1);
        actions.expect_end_heat_wave().returning(|| ()).times(1);

        let mut fsm = super::start(actions);
        // To Summer
        fsm.time_advances(time);
        fsm.time_advances(time);
        // To/Out Scorching
        fsm.temperature_rises(());
        fsm.temperature_drops(());
    }
}
