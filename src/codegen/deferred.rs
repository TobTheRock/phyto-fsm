use crate::fsm;

use super::ident;

pub struct DeferredEventsCodegen {
    pub state_field: proc_macro2::TokenStream,
    pub state_clone_field: proc_macro2::TokenStream,
    pub fsm_field: proc_macro2::TokenStream,
    pub fsm_init_field: proc_macro2::TokenStream,
    pub entry_method: proc_macro2::Ident,
    pub entry_point: proc_macro2::TokenStream,
    event_enum: Option<proc_macro2::Ident>,
}

impl DeferredEventsCodegen {
    pub fn new(fsm: &fsm::UmlFsm, idents: &ident::Idents) -> Self {
        let has_deferred = fsm.states().any(|s| s.deferred_events().next().is_some());
        if has_deferred {
            Self::enabled(idents)
        } else {
            Self::disabled(idents)
        }
    }

    fn disabled(idents: &ident::Idents) -> Self {
        let event_enum = &idents.event_enum;
        Self {
            state_field: quote::quote! {},
            state_clone_field: quote::quote! {},
            fsm_field: quote::quote! {},
            fsm_init_field: quote::quote! {},
            entry_method: quote::format_ident!("trigger_event"),
            entry_point: quote::quote! {
                fn trigger_event(&mut self, event: #event_enum<A>) {
                    self.try_event_based_transition(event);
                    self.try_direct_transition();
                }
            },
            event_enum: None,
        }
    }

    fn enabled(idents: &ident::Idents) -> Self {
        let event_enum = &idents.event_enum;
        Self {
            state_field: quote::quote! { defer_event: fn(event: &#event_enum<A>) -> bool, },
            state_clone_field: quote::quote! { defer_event: self.defer_event, },
            fsm_field: quote::quote! { deferred_events: std::collections::VecDeque<#event_enum<A>>, },
            fsm_init_field: quote::quote! { deferred_events: std::collections::VecDeque::new(), },
            entry_method: quote::format_ident!("run_event_loop"),
            entry_point: quote::quote! {
                fn run_event_loop(&mut self, event: #event_enum<A>) {
                    let pending = std::mem::take(&mut self.deferred_events);
                    std::iter::once(event)
                        .chain(pending.into_iter())
                        .for_each(|event| {
                            self.process_event(event);
                        });
                }

                fn process_event(&mut self, event: #event_enum<A>) {
                    let Some(event) = self.try_defer_event(event) else {
                        return;
                    };
                    if self.try_event_based_transition(event) {
                        self.try_direct_transition();
                    }
                }

                fn try_defer_event(&mut self, event: #event_enum<A>) -> Option<#event_enum<A>> {
                    if self.current_state.defer_event(&event) {
                        self.deferred_events.push_back(event);
                        None
                    } else {
                        Some(event)
                    }
                }
            },
            event_enum: Some(event_enum.clone()),
        }
    }

    pub fn state_node_defer_event_method(
        &self,
        state_node: &proc_macro2::Ident,
    ) -> proc_macro2::TokenStream {
        let event_enum = match &self.event_enum {
            Some(e) => e,
            None => return quote::quote! {},
        };
        quote::quote! {
            fn defer_event(&self, event: &#event_enum<A>) -> bool {
                match self {
                    #state_node::Real(real) => (real.defer_event)(event),
                    #state_node::Initial { .. } => false,
                    #state_node::Exit() => false,
                }
            }
        }
    }

    pub fn state_field_value(&self, state: &fsm::State<'_>) -> proc_macro2::TokenStream {
        let event_enum = match &self.event_enum {
            Some(e) => e,
            None => return quote::quote! {},
        };

        let deferred: Vec<_> = state.deferred_events().collect();
        let defer_fn = if deferred.is_empty() {
            quote::quote! { |_event| false }
        } else {
            let match_arms = deferred.iter().map(|event| {
                let event_ident = event.ident();
                quote::quote! { #event_enum::#event_ident(_) => true, }
            });
            quote::quote! {
                |event| match event {
                    #(#match_arms)*
                    _ => false,
                }
            }
        };

        quote::quote! { defer_event: #defer_fn, }
    }
}
