use super::{GenerationContext, extract};

pub fn generate_event_params_trait(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let trait_ident = &ctx.idents.event_params_trait;
    let associated_types = extract::events(ctx.fsm).map(|event| {
        let type_ident = event.params_ident();
        quote::quote! { type #type_ident; }
    });

    quote::quote! {
        pub trait #trait_ident {
            #(#associated_types)*
        }
    }
}

pub fn generate_action_trait(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let action_methods = extract::actions(ctx.fsm).map(|(action, event)| {
        let action_ident = action.ident();
        let params_ident = event.params_ident();
        quote::quote! {
            fn #action_ident(&mut self, params: Self::#params_ident);
        }
    });

    let guard_methods = extract::guards(ctx.fsm).map(|(guard, event)| {
        let guard_ident = guard.ident();
        let params_ident = event.params_ident();
        quote::quote! {
            fn #guard_ident(&self, event: &Self::#params_ident) -> bool;
        }
    });

    let direct_action_methods = extract::direct_transition_actions(ctx.fsm).map(|action| {
        let action_ident = action.ident();
        quote::quote! {
            fn #action_ident(&mut self);
        }
    });

    let direct_guard_methods = extract::direct_transition_guards(ctx.fsm).map(|guard| {
        let guard_ident = guard.ident();
        quote::quote! {
            fn #guard_ident(&self) -> bool;
        }
    });

    let enter_methods = extract::enter_actions(ctx.fsm).map(|action| {
        let action_ident = action.ident();
        quote::quote! {
            fn #action_ident(&mut self);
        }
    });

    let exit_methods = extract::exit_actions(ctx.fsm).map(|action| {
        let action_ident = action.ident();
        quote::quote! {
            fn #action_ident(&mut self);
        }
    });

    let event_params_trait = &ctx.idents.event_params_trait;
    let trait_ident = &ctx.idents.action_trait;

    quote::quote! {
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
    let init_state_id_variant = &ctx.idents.init_state_id_variant;

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
        enum #state_id_enum {
            #(#variants)*
            #init_state_id_variant,
        }

        impl From<#state_id_enum> for &'static str {
            fn from(id: #state_id_enum) -> Self {
                match id {
                    #(#from_match_arms)*
                    #state_id_enum::#init_state_id_variant => "[*]",
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
    let state_ident = &ctx.idents.state_struct;
    let state_id_enum = &ctx.idents.state_id_enum;
    let actions_trait = &ctx.idents.action_trait;
    let event_enum = &ctx.idents.event_enum;

    let defer_field = &ctx.deferred.state_field;
    let defer_clone = &ctx.deferred.state_clone_field;

    quote::quote! {
        #[derive(Copy)]
        struct #state_ident<A: #actions_trait> {
            id: #state_id_enum,
            transition: fn(event: #event_enum<A>, actions: &mut A) -> Option<Self>,
            direct_transition: fn(actions: &mut A) -> Option<Self>,
            enter_state: fn() -> Self,
            enter: fn(&mut A, from: &Self),
            exit: fn(&mut A, to: &Self),
            #defer_field
        }

        impl<A: #actions_trait> Clone for #state_ident<A> {
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

        impl<A: #actions_trait> PartialEq for #state_ident<A> {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
    }
}

pub fn generate_state_impl(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let state_id_enum = &ctx.idents.state_id_enum;
    let init_state_id_variant = &ctx.idents.init_state_id_variant;
    let fsm_enter_fn = ctx.fsm.enter_state().function_ident();

    let state_fns = ctx.fsm.states().map(|state| {
        let state_id_variant = state.state_id_variant_ident();
        let fn_name = state.function_ident();

        let transitions = state.transitions().filter_map(|t| {
            let event_ident = t.event?.ident();
            let event_enum = &ctx.idents.event_enum;
            let next_state = t
                .destination
                .as_ref()
                .map(|d| {
                    let fn_ident = d.function_ident();
                    quote::quote! { Some(Self::#fn_ident()) }
                })
                .unwrap_or_else(|| quote::quote! { None });
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
                    {
                    let parent = Self::#parent_fn();
                    (parent.transition)(event, action)
                }
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

        quote::quote! {
            fn #fn_name() -> Self {
                Self {
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
                }
            }
        }
    });

    let struct_ident = &ctx.idents.state_struct;
    let actions_trait = &ctx.idents.action_trait;
    let init_defer = &ctx.deferred.state_init_field;
    quote::quote! {
        impl<A: #actions_trait> #struct_ident<A> {
            fn init() -> Self {
                Self {
                    id: #state_id_enum::#init_state_id_variant,
                    transition: |_event, _action| None,
                    direct_transition: |_action| Some(Self::#fsm_enter_fn()),
                    enter_state: Self::init,
                    enter: |_actions, _from| {},
                    exit: |_actions, _to| {},
                    #init_defer
                }
            }

            #(#state_fns)*
        }
    }
}

pub fn generate_fsm(ctx: &GenerationContext) -> proc_macro2::TokenStream {
    let fsm = &ctx.idents.fsm;
    let fsm_inner = &ctx.idents.fsm_inner;
    let action = &ctx.idents.action_trait;
    let state = &ctx.idents.state_struct;
    let event_enum = &ctx.idents.event_enum;
    let event_params_trait = &ctx.idents.event_params_trait;

    let deferred_field = &ctx.deferred.fsm_field;
    let deferred_init = &ctx.deferred.fsm_init_field;

    let fsm_struct = quote::quote! {
        struct #fsm_inner<A: #action> {
            actions: A,
            current_state: #state<A>,
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
        quote::quote! {
            pub fn #fn_ident(&mut self, params: <A as #event_params_trait>::#params_ident) {
                self.0.#entry_method(#event_enum::#event_ident(params));
            }
        }
    });

    let common_impl = quote::quote! {
        impl<A> #fsm_inner<A>
        where
            A: #action,
        {
            fn start(actions: A) -> Self {
                let mut fsm = Self {
                    actions,
                    current_state: #state::init(),
                    #deferred_init
                };
                fsm.try_direct_transition();
                fsm
            }

            fn change_state(&mut self, next_state: #state<A>) {
                (self.current_state.exit)(&mut self.actions, &next_state);
                (next_state.enter)(&mut self.actions, &self.current_state);
                self.current_state = next_state;
            }

            fn try_direct_transition(&mut self) {
                while let Some(transition_state) = (self.current_state.direct_transition)(&mut self.actions) {
                    let enter_state = (transition_state.enter_state)();
                    self.change_state(enter_state);
                }
            }
        }

        impl<A> #fsm<A>
        where
            A: #action,
        {
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
            if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
                let enter_state = (transition_state.enter_state)();
                ::log::log!(#level, #log_transition,
                    self.current_state.id,
                    event_name,
                    transition_state.id,
                    enter_state.id
                );
                self.change_state(enter_state);
                return true;
            }
            false
        }
    } else {
        quote::quote! {
            if let Some(transition_state) = (self.current_state.transition)(event, &mut self.actions) {
                let enter_state = (transition_state.enter_state)();
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
    let direct_transitions: Vec<_> = state
        .transitions()
        .filter(|t| t.event.is_none() && t.destination.is_some())
        .collect();

    if direct_transitions.is_empty() {
        return quote::quote! { |_action| None };
    }

    let all_guarded = direct_transitions.iter().all(|t| t.guard.is_some());

    let branches: Vec<_> = direct_transitions
        .iter()
        .map(|t| {
            let dest = t.destination.as_ref().unwrap();
            let dest_fn = dest.function_ident();

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
                        return Some(Self::#dest_fn());
                    }
                }
            } else {
                quote::quote! {
                    #action
                    return Some(Self::#dest_fn());
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
        (Self::#parent_fn().enter)(actions, from);
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
        (Self::#parent_fn().exit)(actions, to);
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
            if matches!(#check.id, #(#substate_ids)|*) {
                return;
            }
        }
    }
}
