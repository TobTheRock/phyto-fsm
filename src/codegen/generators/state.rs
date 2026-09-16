use crate::codegen::GenerationContext;

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
            let event_ident = t.event()?.ident();
            let event_enum = &ctx.idents.event_enum;
            let next_state = if matches!(t, crate::fsm::Transition::Final { .. }) {
                quote::quote! { Some(Self::Exit()) }
            } else {
                t.destination()
                    .map(|d| {
                        let fn_ident = d.function_ident();
                        quote::quote! { Some(Self::#fn_ident()) }
                    })
                    .unwrap_or_else(|| quote::quote! { None })
            };
            let action = if let Some(a) = t.action() {
                let action_ident = a.ident();
                quote::quote! { action.#action_ident(params); }
            } else {
                quote::quote! {}
            };

            let guard_condition = if let Some(g) = t.guard() {
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

fn generate_direct_transition(state: &crate::fsm::State<'_>) -> proc_macro2::TokenStream {
    // TODO(guarded-parent-direct): unlike event transitions, a substate does NOT inherit its
    // parent's direct transitions (no `_ => Self::parent().direct_transition(action)` fallback).
    // That is deliberate for *completion* transitions (unguarded `Parent --> Done`), which must
    // not auto-fire before the region runs. But a *guarded* parent direct (`Parent --[g]--> X`)
    // is a real UML group/boundary transition and should apply to every substate. Add that
    // fallback for guarded directs only, once a puml needs it. See exit-states notes.
    //
    // Event-less ("completion") transitions: a `Direct` transition to a real state, or a
    // completion exit `S --> [*]` (`Final` with no event) which resolves to the final state.
    let direct_transitions: Vec<_> = state
        .transitions()
        .filter(|t| {
            matches!(
                t,
                crate::fsm::Transition::Direct { .. }
                    | crate::fsm::Transition::Final { event: None, .. }
            )
        })
        .collect();

    if direct_transitions.is_empty() {
        return quote::quote! { |_action| None };
    }

    let all_guarded = direct_transitions.iter().all(|t| t.guard().is_some());

    let branches: Vec<_> = direct_transitions
        .iter()
        .map(|t| {
            let dest = match t {
                crate::fsm::Transition::Final { .. } => quote::quote! { Self::Exit() },
                _ => {
                    let dest_fn = t.destination().unwrap().function_ident();
                    quote::quote! { Self::#dest_fn() }
                }
            };

            let action = if let Some(a) = t.action() {
                let action_ident = a.ident();
                quote::quote! { action.#action_ident(); }
            } else {
                quote::quote! {}
            };

            if let Some(g) = t.guard() {
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
