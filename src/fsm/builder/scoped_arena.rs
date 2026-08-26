use indextree::{Arena, Node, NodeId};
use itertools::Either;

/// A wrapper around indextree::Arena that provides scoped operations.
/// The scope determines the parent context for state lookups and creation.
#[derive(Debug)]
pub struct ScopedArena<T> {
    arena: Arena<T>,
    scope: Option<NodeId>,
}

impl<T> ScopedArena<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            scope: None,
        }
    }

    /// Sets the current scope and returns the previous scope.
    pub fn set_scope(&mut self, scope: Option<NodeId>) -> Option<NodeId> {
        std::mem::replace(&mut self.scope, scope)
    }

    /// Returns the current scope.
    pub fn scope(&self) -> Option<NodeId> {
        self.scope
    }

    /// Creates a new node with the given data in the current scope.
    /// If a scope is set, the node is appended as a child of the scope.
    pub fn new_node_in_scope(&mut self, data: T) -> NodeId {
        let node_id = self.arena.new_node(data);
        if let Some(parent_id) = self.scope {
            parent_id.append(node_id, &mut self.arena);
        }
        node_id
    }

    /// Returns an iterator over nodes that are direct children of the current scope.
    /// If no scope is set, iterates over root nodes.
    pub fn nodes_in_scope(&self) -> impl Iterator<Item = &Node<T>> + '_ {
        match self.scope {
            Some(scope_id) => {
                Either::Left(scope_id.children(&self.arena).map(|id| &self.arena[id]))
            }
            None => Either::Right(self.root_nodes()),
        }
    }

    /// Returns an iterator over all descendant nodes from the current scope.
    /// If no scope is set, iterates from all root nodes.
    pub fn descendants_from_scope(&self) -> impl Iterator<Item = &Node<T>> + '_ {
        let scopes = match self.scope {
            Some(scope_id) => Either::Left(std::iter::once(scope_id)),
            None => Either::Right(self.root_node_ids()),
        };

        scopes
            .flat_map(|id| id.descendants(&self.arena))
            .map(|node_id| &self.arena[node_id])
    }

    /// Returns an iterator over root nodes (nodes without a parent).
    pub fn root_nodes(&self) -> impl Iterator<Item = &Node<T>> + Clone {
        self.arena.iter().filter(|node| node.parent().is_none())
    }

    /// Returns an iterator over root node IDs.
    pub fn root_node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.root_nodes()
            .filter_map(|node| self.arena.get_node_id(node))
    }

    /// Returns an iterator over all node ids
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> {
        self.arena
            .iter()
            .filter_map(|node| self.arena.get_node_id(node))
    }

    /// Consumes the ScopedArena and returns the underlying Arena.
    pub fn into_inner(self) -> Arena<T> {
        self.arena
    }
}

/// Everything that is not scope-aware — `iter`, `get_node_id`, indexing, and the `NodeId`
/// traversals such as `ancestors` — comes straight from the wrapped [`Arena`].
impl<T> std::ops::Deref for ScopedArena<T> {
    type Target = Arena<T>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}

impl<T> std::ops::DerefMut for ScopedArena<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}
