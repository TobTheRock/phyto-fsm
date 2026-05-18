pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid macro input: {0}")]
    InvalidInput(String),
    #[error("Failed to open file {0}: {1}")]
    InvalidFile(String, String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Naming template error: {0}")]
    NamingTemplate(#[from] crate::codegen::naming::NamingError),
}
