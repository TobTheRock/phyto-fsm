use std::collections::HashMap;

use heck::{ToSnakeCase, ToUpperCamelCase};

use crate::file;

const REQUIRED_KEYS: &[&str] = &[
    "fsm",
    "module",
    "event_params_trait",
    "action_trait",
    "state_id_enum",
];

#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum NamingError {
    #[error("Malformed line (expected 'key = value'): {0}")]
    MalformedLine(String),
    #[error("Missing required key: {0}")]
    MissingKey(String),
    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

impl From<NamingError> for crate::error::Error {
    fn from(e: NamingError) -> Self {
        Self::NamingTemplate(e.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct RenderedNames {
    pub fsm: String,
    pub module: String,
    pub event_params_trait: String,
    pub action_trait: String,
    pub state_id_enum: String,
}

#[derive(Debug, Clone)]
pub struct NamingTemplate<'a> {
    content: &'a str,
}

impl Default for NamingTemplate<'static> {
    fn default() -> Self {
        Self {
            content: include_str!("default_naming.tmpl"),
        }
    }
}

impl<'a> From<&'a file::File> for NamingTemplate<'a> {
    fn from(file: &'a file::File) -> Self {
        Self {
            content: file.content(),
        }
    }
}

impl NamingTemplate<'_> {
    pub fn render(&self, name: &str) -> Result<RenderedNames, NamingError> {
        let entries = self.parse_entries()?;
        self.validate_keys(&entries)?;
        Ok(Self::render_entries(&entries, &name.to_upper_camel_case()))
    }

    fn parse_entries(&self) -> Result<HashMap<String, String>, NamingError> {
        let mut entries = HashMap::new();

        for line in self.content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (key, value) = trimmed
                .split_once('=')
                .ok_or_else(|| NamingError::MalformedLine(trimmed.to_string()))?;

            entries.insert(key.trim().to_string(), value.trim().to_string());
        }

        Ok(entries)
    }

    fn validate_keys(&self, entries: &HashMap<String, String>) -> Result<(), NamingError> {
        for key in entries.keys() {
            if !REQUIRED_KEYS.contains(&key.as_str()) {
                return Err(NamingError::UnknownKey(key.clone()));
            }
        }

        for &required in REQUIRED_KEYS {
            if !entries.contains_key(required) {
                return Err(NamingError::MissingKey(required.to_string()));
            }
        }

        Ok(())
    }

    fn render_entries(entries: &HashMap<String, String>, name: &str) -> RenderedNames {
        let render_value = |key: &str| entries[key].replace("{name}", name);

        RenderedNames {
            fsm: render_value("fsm"),
            module: render_value("module").to_snake_case(),
            event_params_trait: render_value("event_params_trait"),
            action_trait: render_value("action_trait"),
            state_id_enum: render_value("state_id_enum"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_on_missing_key() {
        let incomplete = "\
fsm = {name}
module = {name}";
        let template = NamingTemplate {
            content: incomplete,
        };
        let result = template.render("Foo");
        assert!(result.is_err());
    }

    #[test]
    fn error_on_malformed_line() {
        let bad = "\
fsm {name}
module = {name}
event_params_trait = I{name}EventParams
action_trait = I{name}Actions
state_id_enum = {name}StateId";
        let template = NamingTemplate { content: bad };
        let result = template.render("Foo");
        assert!(result.is_err());
    }

    #[test]
    fn error_on_unknown_key() {
        let bad = "\
fsm = {name}
module = {name}
event_params_trait = I{name}EventParams
action_trait = I{name}Actions
state_id_enum = {name}StateId
bogus_key = Something";
        let template = NamingTemplate { content: bad };
        let result = template.render("Foo");
        assert!(result.is_err());
    }

    #[test]
    fn blank_lines_and_whitespace_are_ignored() {
        let spaced = "\
  fsm = {name}

module = {name}
  event_params_trait = I{name}EventParams
action_trait = I{name}Actions
state_id_enum = {name}StateId
";
        let template = NamingTemplate { content: spaced };
        let result = template.render("Foo");
        assert!(result.is_ok());
    }

    #[test]
    fn default_template_renders_correctly() {
        let template = NamingTemplate::default();
        let names = template.render("MyFsm").unwrap();
        assert_eq!(names.fsm, "MyFsm");
        assert_eq!(names.module, "my_fsm");
        assert_eq!(names.event_params_trait, "IMyFsmEventParams");
        assert_eq!(names.action_trait, "IMyFsmActions");
        assert_eq!(names.state_id_enum, "MyFsmState");
    }

    #[test]
    fn every_name_placeholder_is_substituted() {
        let repeated = "\
fsm = {name}{name}
module = {name}
event_params_trait = I{name}EventParams
action_trait = I{name}Actions
state_id_enum = {name}StateId";
        let template = NamingTemplate { content: repeated };
        let names = template.render("Foo").unwrap();
        assert_eq!(names.fsm, "FooFoo");
    }

    #[test]
    fn name_is_converted_to_upper_camel_case() {
        let template = NamingTemplate::default();
        let names = template.render("my_fsm").unwrap();
        assert_eq!(names.fsm, "MyFsm");
        assert_eq!(names.module, "my_fsm");
    }
}
