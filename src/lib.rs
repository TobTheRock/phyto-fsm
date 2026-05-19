use proc_macro::TokenStream;
use quote::quote;

mod codegen;
mod error;
mod file;
mod fsm;
mod logging;
mod options;
mod parser;
#[cfg(test)]
mod test;

/// Generate a finite state machine from a PlantUML state diagram.
///
/// # Parameters
///
/// | Parameter | Description | Default |
/// |-----------|-------------|---------|
/// | **file_path** | Path to the FSM definition file. Required. | — |
/// | **log_level** | Log level for state transitions: `error`, `warn`, `info`, `debug`, `trace`. | None |
/// | **naming** | Path to a custom naming template file. | Built-in default |
///
/// See the [crate-level documentation](index.html) for full details.
#[doc = include_str!("../README.md")]
#[proc_macro]
pub fn generate_fsm(input: TokenStream) -> TokenStream {
    match generate_fsm_inner(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            let error_msg = format!("[phyto-fsm] {}", error);
            quote! {
                compile_error!(#error_msg);
            }
            .into()
        }
    }
}

fn generate_fsm_inner(input: TokenStream) -> error::Result<TokenStream> {
    logging::init();

    let options: options::Options =
        syn::parse(input).map_err(|e| error::Error::InvalidInput(e.to_string()))?;
    let file_path = file::FilePath::resolve(&options.file_path, proc_macro::Span::call_site());
    let file = file::File::try_open(file_path)?;
    let parsed_fsm = fsm::UmlFsm::try_parse(file.content())?;

    let naming_file = options
        .naming_path
        .as_ref()
        .map(|path| {
            let fp = file::FilePath::resolve(path, proc_macro::Span::call_site());
            file::File::try_open(fp)
        })
        .transpose()?;

    let naming = match &naming_file {
        Some(f) => codegen::naming::NamingTemplate::from(f),
        None => codegen::naming::NamingTemplate::default(),
    };

    let codegen_options = codegen::Options {
        naming,
        log_level: options.log_level,
    };

    let fsm_code = codegen::generate(parsed_fsm, codegen_options)?;

    Ok(fsm_code.into())
}
