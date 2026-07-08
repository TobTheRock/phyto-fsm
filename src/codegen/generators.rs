use super::{GenerationContext, extract};

pub fn generate_event_params_trait(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let trait_ident = &ctx.idents.event_params_trait;
    let associated_types = extract::events(ctx.fsm).map(|event| {
        let type_ident = event.params_ident();
        let doc = format!("Parameter type carried by the `{}` event", event.ident());
        quote::quote! {
            #[doc = #doc]
            type #type_ident;
        }
    });

    quote::quote! {
        #[doc = "Associates each FSM event with a user-supplied parameter type"]
        pub trait #trait_ident {
            #(#associated_types)*
        }
    }
}

pub fn generate_action_trait(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let action_methods = extract::actions(ctx.fsm).map(|(action, event)| {
        let action_ident = action.ident();
        let params_ident = event.params_ident();
        let doc = format!("Action triggered by the `{}` event", event.ident());
        quote::quote! {
            #[doc = #doc]
            fn #action_ident(&mut self, params: Self::#params_ident);
        }
    });

    let guard_methods = extract::guards(ctx.fsm).map(|(guard, event)| {
        let guard_ident = guard.ident();
        let params_ident = event.params_ident();
        let doc = format!(
            "Guard for transitions triggered by the `{}` event",
            event.ident()
        );
        quote::quote! {
            #[doc = #doc]
            fn #guard_ident(&self, event: &Self::#params_ident) -> bool;
        }
    });

    let direct_action_methods = extract::direct_transition_actions(ctx.fsm).map(|action| {
        let action_ident = action.ident();
        quote::quote! {
            #[doc = "Action on a direct transition"]
            fn #action_ident(&mut self);
        }
    });

    let direct_guard_methods = extract::direct_transition_guards(ctx.fsm).map(|guard| {
        let guard_ident = guard.ident();
        quote::quote! {
            #[doc = "Guard for direct transitions"]
            fn #guard_ident(&self) -> bool;
        }
    });

    let enter_methods = extract::enter_actions(ctx.fsm).map(|enter| {
        let action_ident = enter.action.ident();
        let doc = format!("Action run when entering {}", doc_state_list(&enter.states));
        quote::quote! {
            #[doc = #doc]
            fn #action_ident(&mut self);
        }
    });

    let exit_methods = extract::exit_actions(ctx.fsm).map(|exit| {
        let action_ident = exit.action.ident();
        let doc = format!("Action run when exiting {}", doc_state_list(&exit.states));
        quote::quote! {
            #[doc = #doc]
            fn #action_ident(&mut self);
        }
    });

    let event_params_trait = &ctx.idents.event_params_trait;
    let trait_ident = &ctx.idents.action_trait;
    let trait_doc = format!(
        "Implement this trait to provide the behavior for `{}`.\n\
         \n\
         The state machine calls into your implementation at every meaningful transition boundary.\n\
         There are four kinds of methods, distinguished by their signature:\n\
         \n\
         - **Transition actions** (`fn foo(&mut self, event: Self::XxxParams)`) — called during an\n\
         \x20 event-triggered transition; the event payload is forwarded to you.\n\
         - **Direct transition actions** (`fn foo(&mut self)`) — called during a guard-driven\n\
         \x20 autonomous transition (no event payload).\n\
         - **Enter / exit actions** (`fn foo(&mut self)`) — called when the machine enters or leaves\n\
         \x20 a particular state.\n\
         - **Guards** (`fn foo(&self, ...) -> bool`) — return `true` to allow a transition, `false`\n\
         \x20 to block it.\n\
         \n\
         Pass your implementation to [`start`] to start the machine.",
        ctx.idents.fsm
    );

    quote::quote! {
        #[doc = #trait_doc]
        pub trait #trait_ident : #event_params_trait{
            #(#action_methods)*
            #(#direct_action_methods)*
            #(#enter_methods)*
            #(#exit_methods)*
            #(#guard_methods)*
            #(#direct_guard_methods)*
        }
    }
}

pub fn generate_event_enum(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let event_variants = extract::events(ctx.fsm).map(|event| {
        let params_ident = event.params_ident();
        let event_ident = event.ident();
        quote::quote! { #event_ident(P::#params_ident),}
    });

    let event_enum_ident = &ctx.idents.event_enum;
    let action_ident = &ctx.idents.action_trait;
    quote::quote! {
        enum #event_enum_ident<P: #action_ident> {
            #(#event_variants)*
        }
    }
}

pub fn generate_event_enum_display(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let event_enum_ident = &ctx.idents.event_enum;
    let event_variants = extract::events(ctx.fsm).map(|event| {
        let event_ident = event.ident();
        let event_name = &event.0;
        quote::quote! { #event_enum_ident::#event_ident(_) => #event_name, }
    });

    let action_ident = &ctx.idents.action_trait;
    quote::quote! {
        impl<P: #action_ident> std::fmt::Display for #event_enum_ident<P> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {
                    #(#event_variants)*
                };
                write!(f, "{}", name)
            }
        }
    }
}

pub fn generate_state_id_enum(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let state_id_enum = &ctx.idents.state_id_enum;

    let variants = ctx.fsm.states().map(|state| {
        let variant_ident = state.state_id_variant_ident();
        quote::quote! { #variant_ident, }
    });

    let from_match_arms = ctx.fsm.states().map(|state| {
        let variant_ident = state.state_id_variant_ident();
        let name_literal = state.name_literal();
        quote::quote! { #state_id_enum::#variant_ident => #name_literal, }
    });

    quote::quote! {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub enum #state_id_enum {
            #(#variants)*
        }

        impl From<#state_id_enum> for &'static str {
            fn from(id: #state_id_enum) -> Self {
                match id {
                    #(#from_match_arms)*
                }
            }
        }

        impl std::fmt::Display for #state_id_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name: &'static str = (*self).into();
                write!(f, "{}", name)
            }
        }
    }
}

pub fn generate_state_struct(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let real_state = &ctx.idents.real_state_struct;
    let state_node = &ctx.idents.state_node_enum;
    let state_id_enum = &ctx.idents.state_id_enum;
    let actions_trait = &ctx.idents.action_trait;
    let event_enum = &ctx.idents.event_enum;
    let fsm_enter_fn = ctx.fsm.enter_state().function_ident();

    let defer_field = &ctx.deferred.state_field;
    let defer_clone = &ctx.deferred.state_clone_field;
    let defer_event_method = ctx.deferred.state_node_defer_event_method(state_node);

    quote::quote! {
        #[derive(Copy)]
        struct #real_state<A: #actions_trait> {
            id: #state_id_enum,
            transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<#state_node<A>>,
            direct_transition: fn(actions: &mut A) -> Option<#state_node<A>>,
            enter_state: fn() -> #state_node<A>,
            enter: fn(&mut A, from: &#state_node<A>),
            exit: fn(&mut A, to: &#state_node<A>),
            #defer_field
        }

        impl<A: #actions_trait> Clone for #real_state<A> {
            fn clone(&self) -> Self {
                Self {
                    id: self.id,
                    transition: self.transition,
                    direct_transition: self.direct_transition,
                    enter_state: self.enter_state,
                    enter: self.enter,
                    exit: self.exit,
                    #defer_clone
                }
            }
        }

        impl<A: #actions_trait> PartialEq for #real_state<A> {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        #[derive(Copy, Clone)]
        enum #state_node<A: #actions_trait> {
            Real(#real_state<A>),
            Initial { target: fn() -> #state_node<A> },
            // The `[*]` final pseudo-state: reaching it ends the FSM (no active state).
            Exit(),
        }

        impl<A: #actions_trait> std::fmt::Display for #state_node<A> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Real(real) => write!(f, "{}", real.id),
                    Self::Initial { .. } => write!(f, "[*]"),
                    Self::Exit() => write!(f, "[*]"),
                }
            }
        }

        impl<A: #actions_trait> #state_node<A> {
            fn init() -> Self {
                Self::Initial { target: Self::#fsm_enter_fn }
            }

            fn id(&self) -> Option<#state_id_enum> {
                match self {
                    Self::Real(real) => Some(real.id),
                    Self::Initial { .. } => None,
                    Self::Exit() => None,
                }
            }

            fn resolve_enter_state(&self) -> Self {
                match self {
                    Self::Real(real) => (real.enter_state)(),
                    Self::Initial { target } => target(),
                    Self::Exit() => Self::Exit(),
                }
            }

            #defer_event_method

            fn transition(&self, event: #event_enum<A>, actions: &mut A) -> Option<Self> {
                match self {
                    Self::Real(real) => (real.transition)(event, actions),
                    Self::Initial { .. } => None,
                    Self::Exit() => None,
                }
            }

            fn direct_transition(&self, actions: &mut A) -> Option<Self> {
                match self {
                    Self::Real(real) => (real.direct_transition)(actions),
                    Self::Initial { target } => Some(target()),
                    Self::Exit() => None,
                }
            }

            fn enter(&self, actions: &mut A, from: &#state_node<A>) {
                match self {
                    Self::Real(real) => (real.enter)(actions, from),
                    Self::Initial { .. } => {}
                    Self::Exit() => {}
                }
            }

            fn exit(&self, actions: &mut A, to: &#state_node<A>) {
                match self {
                    Self::Real(real) => (real.exit)(actions, to),
                    Self::Initial { .. } => {}
                    Self::Exit() => {}
                }
            }
        }
    }
}

pub fn generate_state_impl(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let state_id_enum = &ctx.idents.state_id_enum;
    let real_state = &ctx.idents.real_state_struct;

    let state_fns = ctx.fsm.states().map(|state| {
        let state_id_variant = state.state_id_variant_ident();
        let fn_name = state.function_ident();

        let transitions = state.transitions().filter_map(|t| {
            let event_ident = t.event?.ident();
            let event_enum = &ctx.idents.event_enum;
            let next_state = match &t.target {
                crate::fsm::Target::State(d) => {
                    let fn_ident = d.function_ident();
                    quote::quote! { Some(Self::#fn_ident()) }
                }
                crate::fsm::Target::Internal => quote::quote! { None },
                crate::fsm::Target::Final => quote::quote! { Some(Self::Exit()) },
            };
            let action = if let Some(a) = t.action {
                let action_ident = a.ident();
                quote::quote! { action.#action_ident(params); }
            } else {
                quote::quote! {}
            };

            let guard_condition = if let Some(g) = t.guard {
                let guard_ident = g.ident();
                quote::quote! { if action.#guard_ident(&params) }
            } else {
                quote::quote! {}
            };

            Some(quote::quote! {
                #event_enum::#event_ident(params) #guard_condition => {
                    #action
                    #next_state
                }
            })
        });

        let parent_transition = if let Some(parent) = state.parent() {
            let parent_fn = parent.function_ident();
            quote::quote! {
                Self::#parent_fn().transition(event, action)
            }
        } else {
            quote::quote! {
                None
            }
        };

        let enter_state = state.enter_state();
        let enter_fn = enter_state.function_ident();
        let enter_action = generate_enter_action(&state, state_id_enum);
        let exit_action = generate_exit_action(&state, state_id_enum);
        let direct_transition = generate_direct_transition(&state);
        let defer_event = ctx.deferred.state_field_value(&state);

        // Constructors live on the node type so a transition can yield a real state or a
        // pseudo-state (e.g. `Exit`); each lifts its `RealState` into a node.
        quote::quote! {
            fn #fn_name() -> Self {
                Self::Real(#real_state::<A> {
                    id: #state_id_enum::#state_id_variant,
                    transition: |event, action| match event {
                        #(#transitions,)*
                        _ => #parent_transition,
                    },
                    direct_transition: #direct_transition,
                    enter_state: Self::#enter_fn,
                    enter: #enter_action,
                    exit: #exit_action,
                    #defer_event
                })
            }
        }
    });

    let state_node = &ctx.idents.state_node_enum;
    let actions_trait = &ctx.idents.action_trait;
    quote::quote! {
        impl<A: #actions_trait> #state_node<A> {
            #(#state_fns)*
        }
    }
}

pub fn generate_fsm(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let fsm = &ctx.idents.fsm;
    let fsm_inner = &ctx.idents.fsm_inner;
    let action = &ctx.idents.action_trait;
    let state_node = &ctx.idents.state_node_enum;
    let state_id_enum = &ctx.idents.state_id_enum;
    let event_enum = &ctx.idents.event_enum;
    let event_params_trait = &ctx.idents.event_params_trait;

    let deferred_field = &ctx.deferred.fsm_field;
    let deferred_init = &ctx.deferred.fsm_init_field;

    let fsm_struct = quote::quote! {
        struct #fsm_inner<A: #action> {
            actions: A,
            current_state: #state_node<A>,
            #deferred_field
        }
        pub struct #fsm<A: #action>(#fsm_inner<A>);
    };

    let trigger_event = generate_trigger_event(ctx);

    let entry_method = &ctx.deferred.entry_method;

    let methods = extract::events(ctx.fsm).map(|event| {
        let fn_ident = event.method_ident();
        let event_ident = event.ident();
        let params_ident = event.params_ident();
        let doc = format!("Triggers a `{}` event", event.ident());
        quote::quote! {
            #[doc = #doc]
            pub fn #fn_ident(&mut self, params: <A as #event_params_trait>::#params_ident) {
                self.0.#entry_method(#event_enum::#event_ident(params));
            }
        }
    });

    let direct_transition_body = if let Some(log_level) = ctx.log_level {
        let level = log_level_token(log_level);
        let log_transition = format!("{}: {{}} -[direct]-> {{}}, entering {{}}", ctx.fsm.name());
        quote::quote! {
            while let Some(transition_state) = self.current_state.direct_transition(&mut self.actions) {
                let enter_state = transition_state.resolve_enter_state();
                ::log::log!(#level, #log_transition,
                    self.current_state,
                    transition_state,
                    enter_state
                );
                self.change_state(enter_state);
            }
        }
    } else {
        quote::quote! {
            while let Some(transition_state) = self.current_state.direct_transition(&mut self.actions) {
                let enter_state = transition_state.resolve_enter_state();
                self.change_state(enter_state);
            }
        }
    };

    let common_impl = quote::quote! {
        impl<A> #fsm_inner<A>
        where
            A: #action,
        {
            fn start(actions: A) -> Self {
                let mut fsm = Self {
                    actions,
                    current_state: #state_node::init(),
                    #deferred_init
                };
                fsm.try_direct_transition();
                fsm
            }

            fn change_state(&mut self, next_state: #state_node<A>) {
                self.current_state.exit(&mut self.actions, &next_state);
                next_state.enter(&mut self.actions, &self.current_state);
                self.current_state = next_state;
            }

            fn try_direct_transition(&mut self) {
                #direct_transition_body
            }
        }

        impl<A> #fsm<A>
        where
            A: #action,
        {
            #[doc = "Returns the currently active state. If the FSM was not started or has ended None is returned."]
            pub fn active_state(&self) -> Option<#state_id_enum> {
                self.0.current_state.id()
            }

            #(#methods)*
        }

        pub fn start<A: #action>(actions: A) -> #fsm<A> {
            #fsm(#fsm_inner::start(actions))
        }
    };

    quote::quote! {
        #fsm_struct
        #common_impl
        #trigger_event
    }
}

fn generate_trigger_event(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let fsm_inner = &ctx.idents.fsm_inner;
    let action = &ctx.idents.action_trait;
    let event_enum = &ctx.idents.event_enum;

    let event_body = if let Some(log_level) = ctx.log_level {
        let level = log_level_token(log_level);
        let log_transition = format! {"{}: {{}} -[{{}}]-> {{}}, entering {{}}", ctx.fsm.name()};
        quote::quote! {
            let event_name = format!("{}", event);
            if let Some(transition_state) = self.current_state.transition(event, &mut self.actions) {
                let enter_state = transition_state.resolve_enter_state();
                ::log::log!(#level, #log_transition,
                    self.current_state,
                    event_name,
                    transition_state,
                    enter_state
                );
                self.change_state(enter_state);
                return true;
            }
            false
        }
    } else {
        quote::quote! {
            if let Some(transition_state) = self.current_state.transition(event, &mut self.actions) {
                let enter_state = transition_state.resolve_enter_state();
                self.change_state(enter_state);
                return true;
            }
            false
        }
    };

    let entry_point = &ctx.deferred.entry_point;

    quote::quote! {
        impl<A> #fsm_inner<A>
        where
            A: #action,
        {
            #entry_point

            fn try_event_based_transition(&mut self, event: #event_enum<A>) -> bool {
                #event_body
            }
        }
    }
}

/// Renders the owning states of an enter/exit action for a doc string, backtick-quoted and
/// joined with `or` so a reused action lists every state it fires for.
fn doc_state_list(states: &[String]) -> String {
    states
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(" or ")
}

fn log_level_token(level: log::Level) -> proc_macro2::TokenStream {
    match level {
        log::Level::Error => quote::quote! {log::Level::Error},
        log::Level::Warn => quote::quote! {log::Level::Warn},
        log::Level::Info => quote::quote! {log::Level::Info},
        log::Level::Debug => quote::quote! {log::Level::Debug},
        log::Level::Trace => quote::quote! {log::Level::Trace},
    }
}

fn generate_direct_transition(state: &crate::fsm::State<'_>) -> proc_macro2::TokenStream {
    // Direct (event-less) transitions that actually go somewhere: a real state or `[*]`.
    let direct_transitions: Vec<_> = state
        .transitions()
        .filter(|t| t.event.is_none() && !matches!(t.target, crate::fsm::Target::Internal))
        .collect();

    if direct_transitions.is_empty() {
        return quote::quote! { |_action| None };
    }

    let all_guarded = direct_transitions.iter().all(|t| t.guard.is_some());

    let branches: Vec<_> = direct_transitions
        .iter()
        .map(|t| {
            let dest = match &t.target {
                crate::fsm::Target::State(d) => {
                    let dest_fn = d.function_ident();
                    quote::quote! { Self::#dest_fn() }
                }
                crate::fsm::Target::Final => quote::quote! { Self::Exit() },
                // Filtered out above.
                crate::fsm::Target::Internal => unreachable!(),
            };

            let action = if let Some(a) = t.action {
                let action_ident = a.ident();
                quote::quote! { action.#action_ident(); }
            } else {
                quote::quote! {}
            };

            if let Some(g) = t.guard {
                let guard_ident = g.ident();
                quote::quote! {
                    if action.#guard_ident() {
                        #action
                        return Some(#dest);
                    }
                }
            } else {
                quote::quote! {
                    #action
                    return Some(#dest);
                }
            }
        })
        .collect();

    let fallback = if all_guarded {
        quote::quote! { None }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        |action| {
            #(#branches)*
            #fallback
        }
    }
}

fn generate_enter_action(
    state: &crate::fsm::State<'_>,
    state_id_enum: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let enter_action = if let Some(action) = state.enter_action() {
        let action_ident = action.ident();
        quote::quote! {
            actions.#action_ident();
        }
    } else {
        quote::quote! {}
    };
    let internal_guard = generate_internal_transition_guard(state, state_id_enum, true);
    let parent_enter = if let Some(parent) = state.parent() {
        let parent_fn = parent.function_ident();
        quote::quote! {
        Self::#parent_fn().enter(actions, from);
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        |actions, from|
        {
        #internal_guard
        #parent_enter
        #enter_action
        }
    }
}

fn generate_exit_action(
    state: &crate::fsm::State<'_>,
    state_id_enum: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let exit_action = if let Some(action) = state.exit_action() {
        let action_ident = action.ident();
        quote::quote! {
            actions.#action_ident();
        }
    } else {
        quote::quote! {}
    };
    let internal_guard = generate_internal_transition_guard(state, state_id_enum, false);
    let parent_exit = if let Some(parent) = state.parent() {
        let parent_fn = parent.function_ident();
        quote::quote! {
        Self::#parent_fn().exit(actions, to);
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        |actions, to|
        {
        #internal_guard
        #exit_action
        #parent_exit
        }
    }
}

fn all_substate_ids(
    state: &crate::fsm::State<'_>,
    state_id_enum: &proc_macro2::Ident,
) -> Vec<proc_macro2::TokenStream> {
    state
        .substates()
        .map(|s| {
            let variant = s.state_id_variant_ident();
            quote::quote! { #state_id_enum::#variant }
        })
        .collect()
}

fn generate_internal_transition_guard(
    state: &crate::fsm::State<'_>,
    state_id_enum: &proc_macro2::Ident,
    is_enter: bool,
) -> proc_macro2::TokenStream {
    let substate_ids = all_substate_ids(state, state_id_enum);
    if substate_ids.is_empty() {
        quote::quote! {}
    } else {
        let check = if is_enter {
            quote::quote! {from}
        } else {
            quote::quote! {to}
        };
        quote::quote! {
            if matches!(#check.id(), Some(#(#substate_ids)|*)) {
                return;
            }
        }
    }
}
