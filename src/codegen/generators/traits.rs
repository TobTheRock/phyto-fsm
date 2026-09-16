use crate::codegen::{GenerationContext, extract};

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

/// Renders the owning states of an enter/exit action for a doc string, backtick-quoted and
/// joined with `or` so a reused action lists every state it fires for.
fn doc_state_list(states: &[String]) -> String {
    states
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(" or ")
}
