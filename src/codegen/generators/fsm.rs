use crate::codegen::{GenerationContext, extract};

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

    let log_stmt = ctx.log_level.map(|log_level| {
        let level = log_level_token(log_level);
        let log_transition = format!("{}: {{}} -[direct]-> {{}}, entering {{}}", ctx.fsm.name());
        quote::quote! {
            ::log::log!(#level, #log_transition, self.current_state, transition_state, enter_state);
        }
    });

    let direct_transition_body = quote::quote! {
        while let Some(transition_state) = self.current_state.direct_transition(&mut self.actions) {
            let enter_state = transition_state.resolve_enter_state();
            #log_stmt
            self.change_state(enter_state);
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

    // Two holes: the name must be captured before `transition` consumes the event, the log call
    // needs the transition's outcome.
    let (capture_event_name, log_stmt) = match ctx.log_level {
        Some(log_level) => {
            let level = log_level_token(log_level);
            let log_transition = format!("{}: {{}} -[{{}}]-> {{}}, entering {{}}", ctx.fsm.name());
            (
                quote::quote! { let event_name = format!("{}", event); },
                quote::quote! {
                    ::log::log!(#level, #log_transition,
                        self.current_state, event_name, transition_state, enter_state);
                },
            )
        }
        None => Default::default(),
    };

    let event_body = quote::quote! {
        #capture_event_name
        if let Some(transition_state) = self.current_state.transition(event, &mut self.actions) {
            let enter_state = transition_state.resolve_enter_state();
            #log_stmt
            self.change_state(enter_state);
            return true;
        }
        false
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
