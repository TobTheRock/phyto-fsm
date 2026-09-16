use syn::{
    LitStr,
    parse::{Parse, ParseStream},
};

pub struct Options {
    pub file_path: String,
    pub sub_fsms: Vec<String>,
    pub naming_path: Option<String>,
    pub log_level: Option<log::Level>,
}

impl Options {
    fn try_from_file_path(lit: &LitStr) -> syn::Result<Self> {
        Ok(Self {
            file_path: parse_non_empty(lit, "File path")?,
            sub_fsms: Vec::new(),
            naming_path: None,
            log_level: None,
        })
    }

    fn try_from_key_value_pairs(input: ParseStream) -> syn::Result<Self> {
        let parsed_pairs =
            syn::punctuated::Punctuated::<OptionKeyValue, syn::Token![,]>::parse_terminated(input)?;

        let mut file_path = None;
        let mut sub_fsms = None;
        let mut naming_path = None;
        let mut log_level = None;

        for pair in parsed_pairs {
            let (key, was_set) = match pair {
                OptionKeyValue::FilePath(path) => ("file_path", file_path.replace(path).is_some()),
                OptionKeyValue::SubFsms(paths) => ("sub_fsms", sub_fsms.replace(paths).is_some()),
                OptionKeyValue::Naming(path) => ("naming", naming_path.replace(path).is_some()),
                OptionKeyValue::LogLevel(level) => {
                    ("log_level", log_level.replace(level).is_some())
                }
            };
            if was_set {
                return Err(syn::Error::new(
                    input.span(),
                    format!("Expected at most one '{key}' key in options"),
                ));
            }
        }

        Ok(Self {
            file_path: file_path.ok_or_else(|| {
                syn::Error::new(input.span(), "Expected a 'file_path' key in options")
            })?,
            sub_fsms: sub_fsms.unwrap_or_default(),
            naming_path,
            log_level,
        })
    }
}

fn parse_non_empty(lit: &LitStr, what: &str) -> syn::Result<String> {
    let value = lit.value();
    if value.trim().is_empty() {
        return Err(syn::Error::new(
            lit.span(),
            format!("{what} cannot be empty"),
        ));
    }
    Ok(value)
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
    SubFsms(Vec<String>),
    LogLevel(log::Level),
    Naming(String),
}

impl Parse for OptionKeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        match key.to_string().as_str() {
            "file_path" => Ok(OptionKeyValue::FilePath(parse_non_empty(
                &input.parse()?,
                "File path",
            )?)),
            "sub_fsms" => {
                let content;
                syn::bracketed!(content in input);
                let paths =
                    syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated(
                        &content,
                    )?;
                Ok(OptionKeyValue::SubFsms(
                    paths.iter().map(|lit| lit.value()).collect(),
                ))
            }
            "log_level" => {
                let lit: LitStr = input.parse()?;
                let level_str = lit.value();
                let log_level = parse_log_level(&level_str, lit.span())?;
                Ok(OptionKeyValue::LogLevel(log_level))
            }
            "naming" => Ok(OptionKeyValue::Naming(parse_non_empty(
                &input.parse()?,
                "Naming template path",
            )?)),
            _ => Err(syn::Error::new(
                key.span(),
                "Unknown option key. Expected 'file_path', 'sub_fsms', 'log_level', or 'naming'",
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
    fn error_on_duplicate_optional_keys() {
        let duplicates = [
            quote::quote!(
                file_path = "fsm.puml",
                log_level = "error",
                log_level = "warn"
            ),
            quote::quote!(file_path = "fsm.puml", naming = "a.tmpl", naming = "b.tmpl"),
            quote::quote!(
                file_path = "fsm.puml",
                sub_fsms = ["a.puml"],
                sub_fsms = ["b.puml"]
            ),
        ];
        for tokens in duplicates {
            assert!(Options::parse.parse2(tokens).is_err());
        }
    }

    #[test]
    fn error_on_missing_file_path() {
        let tokens = quote::quote!(log_level = "error");
        assert!(Options::parse.parse2(tokens).is_err());
    }

    #[test]
    fn error_on_empty_naming_path() {
        let tokens = quote::quote!(file_path = "fsm.puml", naming = "   ");
        assert!(Options::parse.parse2(tokens).is_err());
    }

    #[test]
    fn parse_sub_fsms() {
        let tokens = quote::quote!(file_path = "fsm.puml", sub_fsms = ["a.puml", "b.puml"]);
        let options = Options::parse.parse2(tokens).unwrap();
        assert_eq!(options.sub_fsms, ["a.puml", "b.puml"]);
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
