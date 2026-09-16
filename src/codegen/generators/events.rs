use crate::codegen::{GenerationContext, extract};

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
