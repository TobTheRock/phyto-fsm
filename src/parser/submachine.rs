use std::ops::{Deref, DerefMut};

use crate::error::{Error, Result};

use super::plantuml;

/// Maps a sub-FSM's `@startuml` name to its elements, for resolving submachine references
/// (`state X : SubName`).
type SubMachines<'a> = std::collections::HashMap<&'a str, &'a plantuml::StateElements<'a>>;

/// Resolves `state X : SubName` descriptions against a pool of named sub-FSMs and rejects
/// reference cycles. Owns the *submachine linking* concern, keeping it out of the translation
/// walk. The plain parse uses [`SubMachineResolver::none`], which resolves nothing.
pub struct SubMachineResolver<'a> {
    machines: SubMachines<'a>,
    visiting: Vec<&'a str>,
}

impl<'a> SubMachineResolver<'a> {
    /// A resolver with no sub-FSMs; every description is a plain state description.
    pub fn none() -> Self {
        Self {
            machines: SubMachines::new(),
            visiting: Vec::new(),
        }
    }

    /// Builds a resolver from parsed sub-FSMs, keyed by their `@startuml` name.
    pub fn try_from_subs(diagrams: &'a [plantuml::StateDiagram<'a>]) -> Result<Self> {
        let entries = diagrams
            .iter()
            .map(|diagram| named_entry(diagram.name(), diagram.elements()))
            .collect::<Result<Vec<_>>>()?;
        Self::from_entries(entries)
    }

    /// Parser-agnostic core: builds a resolver from `(name, elements)` entries, rejecting duplicate
    /// names. Never inspects the elements — only stores them for [`enter`](Self::enter) to hand
    /// back — so it can be exercised without going through the parser.
    fn from_entries(
        entries: impl IntoIterator<Item = (&'a str, &'a plantuml::StateElements<'a>)>,
    ) -> Result<Self> {
        let mut machines = SubMachines::new();
        for (name, elements) in entries {
            if machines.insert(name, elements).is_some() {
                return Err(Error::Parse(format!("duplicate sub-FSM name '{name}'")));
            }
        }
        Ok(Self {
            machines,
            visiting: Vec::new(),
        })
    }

    /// Whether `name` refers to a sub-FSM rather than a plain state description.
    pub fn is_ref(&self, name: &str) -> bool {
        self.machines.contains_key(name)
    }

    /// If `name` refers to a sub-FSM, marks it in-flight and returns a [`SubMachineGuard`] that
    /// pops it when dropped. Recurse through the guard (it derefs to the resolver), then let it
    /// drop. Errors on a cycle.
    pub fn enter(&mut self, name: &'a str) -> Result<Option<SubMachineGuard<'_, 'a>>> {
        let Some(elements) = self.machines.get(name).copied() else {
            return Ok(None);
        };
        if self.visiting.contains(&name) {
            return Err(Error::Parse(format!(
                "cyclic sub-FSM reference: '{name}' is already being expanded"
            )));
        }
        self.visiting.push(name);
        Ok(Some(SubMachineGuard {
            resolver: self,
            elements,
        }))
    }
}

/// Pairs a sub-FSM's `@startuml` name with its elements into a map entry, erroring if the sub is
/// unnamed. Takes the name and elements directly (not a diagram) so it stays parser-agnostic.
fn named_entry<'a>(
    name: Option<&'a str>,
    elements: &'a plantuml::StateElements<'a>,
) -> Result<(&'a str, &'a plantuml::StateElements<'a>)> {
    let name =
        name.ok_or_else(|| Error::Parse("a sub-FSM is missing its @startuml name".to_string()))?;
    Ok((name, elements))
}

/// An in-flight sub-FSM expansion. Holds the expanding sub-FSM's elements and pops it off the
/// resolver's visiting stack on drop, so [`enter`](SubMachineResolver::enter)/leave stay balanced
/// without a manual call. Derefs to the resolver so recursion can keep resolving nested refs.
pub struct SubMachineGuard<'r, 'a> {
    resolver: &'r mut SubMachineResolver<'a>,
    elements: &'a plantuml::StateElements<'a>,
}

impl<'a> SubMachineGuard<'_, 'a> {
    /// The elements of the sub-FSM being expanded.
    pub fn elements(&self) -> &'a plantuml::StateElements<'a> {
        self.elements
    }
}

impl<'a> Deref for SubMachineGuard<'_, 'a> {
    type Target = SubMachineResolver<'a>;

    fn deref(&self) -> &Self::Target {
        self.resolver
    }
}

impl DerefMut for SubMachineGuard<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resolver
    }
}

impl Drop for SubMachineGuard<'_, '_> {
    fn drop(&mut self) {
        self.resolver.visiting.pop();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use plantuml::StateElements;

    // The resolver never inspects elements and `named_entry` takes the name directly, so no test
    // here needs the parser; empty elements stand in for the payload.

    #[test]
    fn rejects_duplicate_sub_names() {
        let elements = StateElements::default();
        let result = SubMachineResolver::from_entries([("Dup", &elements), ("Dup", &elements)]);
        assert!(result.is_err());
    }

    #[test]
    fn named_entry_rejects_unnamed_sub() {
        let elements = StateElements::default();
        let result = named_entry(None, &elements);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_reference_is_not_a_sub() {
        let elements = StateElements::default();
        let mut resolver = SubMachineResolver::from_entries([("Worker", &elements)]).unwrap();
        assert!(!resolver.is_ref("Typo"));
        assert!(matches!(resolver.enter("Typo"), Ok(None)));
    }

    #[test]
    fn rejects_reference_cycle() {
        let elements = StateElements::default();
        let mut resolver = SubMachineResolver::from_entries([("Worker", &elements)]).unwrap();
        let guard = resolver.enter("Worker").expect("Worker resolves");
        assert!(guard.is_some());
        // Re-entering while the first expansion's guard is still alive is the cycle.
        assert!(guard.unwrap().enter("Worker").is_err());
    }
}
