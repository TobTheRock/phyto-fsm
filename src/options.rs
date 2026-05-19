use itertools::Itertools;
use syn::{
    LitStr,
    parse::{Parse, ParseStream},
};

pub struct Options {
    pub file_path: String,
    pub naming_path: Option<String>,
    pub log_level: Option<log::Level>,
}

impl Options {
    fn try_from_file_path(lit: &LitStr) -> syn::Result<Self> {
        let file_path = lit.value();
        if file_path.trim().is_empty() {
            return Err(syn::Error::new(lit.span(), "File path cannot be empty"));
        }
        Ok(Self {
            file_path,
            naming_path: None,
            log_level: None,
        })
    }

    fn try_from_key_value_pairs(input: ParseStream) -> syn::Result<Self> {
        let parsed_pairs =
            syn::punctuated::Punctuated::<OptionKeyValue, syn::Token![,]>::parse_terminated(input)?;
        let file_path = parsed_pairs
            .iter()
            .filter_map(|pair| {
                if let OptionKeyValue::FilePath(path) = pair {
                    Some(path)
                } else {
                    None
                }
            })
            .exactly_one()
            .map_err(|_| {
                syn::Error::new(
                    input.span(),
                    "Expected exactly one 'file_path' key in options",
                )
            })?;

        let log_level = parsed_pairs
            .iter()
            .filter_map(|pair| {
                if let OptionKeyValue::LogLevel(level) = pair {
                    Some(*level)
                } else {
                    None
                }
            })
            .at_most_one()
            .map_err(|_| {
                syn::Error::new(
                    input.span(),
                    "Expected at most one 'log_level' key in options",
                )
            })?;

        let naming_path: Option<String> = parsed_pairs
            .iter()
            .filter_map(|pair| {
                if let OptionKeyValue::Naming(path) = pair {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .at_most_one()
            .map_err(|_| {
                syn::Error::new(input.span(), "Expected at most one 'naming' key in options")
            })?;

        Ok(Self {
            file_path: file_path.clone(),
            naming_path,
            log_level,
        })
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(input.span(), "Expected macro input"));
        }

        if input.peek(syn::LitStr) {
            let file_path: LitStr = input.parse()?;
            return Options::try_from_file_path(&file_path);
        }

        Options::try_from_key_value_pairs(input)
    }
}

enum OptionKeyValue {
    FilePath(String),
    LogLevel(log::Level),
    Naming(String),
}

impl Parse for OptionKeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        match key.to_string().as_str() {
            "file_path" => {
                let lit: LitStr = input.parse()?;
                let file_path = lit.value();
                if file_path.trim().is_empty() {
                    return Err(syn::Error::new(lit.span(), "File path cannot be empty"));
                }
                Ok(OptionKeyValue::FilePath(file_path))
            }
            "log_level" => {
                let lit: LitStr = input.parse()?;
                let level_str = lit.value();
                let log_level = parse_log_level(&level_str, lit.span())?;
                Ok(OptionKeyValue::LogLevel(log_level))
            }
            "naming" => {
                let lit: LitStr = input.parse()?;
                let value = lit.value();
                if value.trim().is_empty() {
                    return Err(syn::Error::new(
                        lit.span(),
                        "Naming template path cannot be empty",
                    ));
                }
                Ok(OptionKeyValue::Naming(value))
            }
            _ => Err(syn::Error::new(
                key.span(),
                "Unknown option key. Expected 'file_path', 'log_level', or 'naming'",
            )),
        }
    }
}

fn parse_log_level(level: &str, span: proc_macro2::Span) -> syn::Result<log::Level> {
    match level.to_lowercase().as_str() {
        "error" => Ok(log::Level::Error),
        "warn" => Ok(log::Level::Warn),
        "info" => Ok(log::Level::Info),
        "debug" => Ok(log::Level::Debug),
        "trace" => Ok(log::Level::Trace),
        _ => Err(syn::Error::new(
            span,
            "Invalid log level. Expected one of: error, warn, info, debug, trace",
        )),
    }
}

#[cfg(test)]
mod test {
    use syn::parse::Parser;

    use super::*;

    fn try_parse_file_path(input: &str) -> syn::Result<Options> {
        let token_stream = quote::quote!(#input);
        Options::parse.parse2(token_stream)
    }

    #[test]
    fn parse_file_path_only() {
        let options = try_parse_file_path("path/to/fsm.puml").unwrap();
        assert_eq!(options.file_path, "path/to/fsm.puml");
        assert_eq!(options.log_level, None);
    }

    #[test]
    fn error_from_empty_file_path() {
        let result = try_parse_file_path("");
        assert!(result.is_err());

        let result = try_parse_file_path("    ");
        assert!(result.is_err());
    }

    #[test]
    fn parse_file_path_as_key_value() {
        let tokens = quote::quote!(file_path = "path/to/fsm.puml");
        let options = Options::parse.parse2(tokens).unwrap();
        assert_eq!(options.file_path, "path/to/fsm.puml");
        assert_eq!(options.log_level, None);
    }

    #[test]
    fn parse_key_value_pairs() {
        let tokens = quote::quote!(file_path = "path/to/fsm.puml", log_level = "error");
        let options = Options::parse.parse2(tokens).unwrap();
        assert_eq!(options.file_path, "path/to/fsm.puml");
        assert_eq!(options.log_level, Some(log::Level::Error));
    }

    #[test]
    fn error_on_duplicate_keys() {
        let tokens = quote::quote!(
            file_path = "path/to/fsm.puml",
            file_path = "another/path.puml"
        );
        let result = Options::parse.parse2(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn error_on_invalid_log_level() {
        let tokens = quote::quote!(file_path = "path/to/fsm.puml", log_level = "INVALID");
        let result = Options::parse.parse2(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn parse_naming_option() {
        let tokens = quote::quote!(
            file_path = "path/to/fsm.puml",
            naming = "path/to/naming.tmpl"
        );
        let options = Options::parse.parse2(tokens).unwrap();
        assert_eq!(options.naming_path, Some("path/to/naming.tmpl".to_string()));
    }
}
