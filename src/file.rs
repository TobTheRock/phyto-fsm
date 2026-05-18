use crate::error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilePath(std::path::PathBuf);

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}

impl FilePath {
    /// Creates a FilePath, resolving relative paths in the following order:
    /// 1. Absolute paths are used as-is
    /// 2. Paths starting with `../` are resolved relative to the caller's location
    /// 3. Other relative paths are resolved relative to `src/` in the manifest directory
    pub fn resolve(file_path: &str, span: proc_macro::Span) -> Self {
        let file_path_str = file_path.trim_matches('"');
        let file_path = std::path::PathBuf::from(file_path_str);

        if file_path.is_absolute() {
            return Self(file_path);
        }

        if is_relative_path(&file_path) {
            return FilePath::resolve_relative_to_caller(file_path_str, span);
        }

        FilePath::resolve_to_src_dir(file_path_str)
    }

    fn resolve_relative_to_caller(file_path: &str, span: proc_macro::Span) -> Self {
        let caller_file = span.local_file().unwrap_or_default();
        let caller_dir = caller_file.parent().unwrap_or(std::path::Path::new("."));
        Self(caller_dir.join(file_path))
    }

    fn resolve_to_src_dir(file_path: &str) -> Self {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let src_path = std::path::PathBuf::from(manifest_dir)
            .join("src")
            .join(file_path);
        Self(src_path)
    }
}

fn is_relative_path(file_path: &std::path::Path) -> bool {
    file_path.starts_with("../") || file_path.starts_with("./")
}

pub struct File {
    content: String,
}

impl File {
    pub fn try_open(file_path: FilePath) -> error::Result<Self> {
        let error =
            |e: std::io::Error| error::Error::InvalidFile(file_path.to_string(), e.to_string());
        let content = std::fs::read_to_string(&file_path.0).map_err(error)?;

        Ok(Self { content })
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[cfg(test)]
mod test {
    use crate::{
        file::{File, FilePath},
        test,
    };

    // TODO use procmacro2 and reenable the test
    // #[test]
    // fn test_abs_file_path_from_relative() {
    //     let file_path = file!();
    //     let expected = std::fs::canonicalize(file_path).expect("Failed to canonicalize path");
    //     let span = proc_macro2::Span::call_site();
    //
    //     let abs_file_path =
    //         FilePath::try_resolve(file_path, span).expect("Failed to create AbsFilePath");
    //
    //     assert_eq!(abs_file_path.0, dbg!(expected));
    // }
    //
    #[test]
    fn open_file() {
        let test_data = test::FsmTestData::four_seasons();

        // TODO use the actual method
        let file_path = FilePath(test_data.path);
        let fsm_file = File::try_open(file_path).expect("Failed to open FSM file");
        assert!(
            !fsm_file.content.is_empty(),
            "FSM file content should not be empty"
        );
    }
}
