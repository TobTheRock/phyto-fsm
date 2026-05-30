use heck::{ToSnakeCase, ToUpperCamelCase};

use super::naming::RenderedNames;
use crate::fsm;
use crate::fsm::{Action, Event};

pub struct Idents {
    pub fsm: proc_macro2::Ident,
    pub fsm_inner: proc_macro2::Ident,
    pub module: proc_macro2::Ident,
    pub event_params_trait: proc_macro2::Ident,
    pub event_enum: proc_macro2::Ident,
    pub action_trait: proc_macro2::Ident,
    pub real_state_struct: proc_macro2::Ident,
    pub state_node_enum: proc_macro2::Ident,
    pub state_id_enum: proc_macro2::Ident,
}

impl From<RenderedNames> for Idents {
    fn from(names: RenderedNames) -> Self {
        Self {
            fsm: quote::format_ident!("{}", names.fsm),
            fsm_inner: quote::format_ident!("FsmInner"),
            module: quote::format_ident!("{}", names.module),
            event_params_trait: quote::format_ident!("{}", names.event_params_trait),
            event_enum: quote::format_ident!("Event"),
            action_trait: quote::format_ident!("{}", names.action_trait),
            real_state_struct: quote::format_ident!("RealState"),
            state_node_enum: quote::format_ident!("StateNode"),
            state_id_enum: quote::format_ident!("{}", names.state_id_enum),
        }
    }
}

impl Event {
    pub fn params_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}Params", self.0.to_upper_camel_case())
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_upper_camel_case())
    }

    pub fn method_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_snake_case())
    }
}

impl Action {
    pub fn ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.0.to_snake_case())
    }
}

impl fsm::State<'_> {
    pub fn function_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.qualified_name("_").to_snake_case())
    }

    pub fn state_id_variant_ident(&self) -> proc_macro2::Ident {
        quote::format_ident!("{}", self.qualified_name("").to_upper_camel_case())
    }

    pub fn name_literal(&self) -> proc_macro2::Literal {
        proc_macro2::Literal::string(&self.qualified_name("::"))
    }

    fn qualified_name(&self, separator: impl Into<String>) -> String {
        use itertools::Itertools;
        let names: Vec<_> = std::iter::successors(Some(self.clone()), |next| next.parent())
            .map(|s| s.name().to_string())
            .collect();
        Itertools::intersperse(names.into_iter().rev(), separator.into()).collect()
    }
}

impl quote::ToTokens for fsm::State<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name().to_tokens(tokens);
    }
}
