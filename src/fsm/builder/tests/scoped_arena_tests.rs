use super::super::scoped_arena::ScopedArena;

#[test]
fn new_node_at_root_when_no_scope() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let node = arena.new_node_in_scope("root");

    assert!(arena[node].parent().is_none());
    assert_eq!(arena.root_nodes().count(), 1);
}

#[test]
fn new_node_as_child_when_scoped() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let parent = arena.new_node_in_scope("parent");

    arena.set_scope(Some(parent));
    let child = arena.new_node_in_scope("child");

    assert_eq!(arena[child].parent(), Some(parent));
    assert_eq!(arena.root_nodes().count(), 1);
}

#[test]
fn ancestors_start_at_the_node_itself() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let parent = arena.new_node_in_scope("parent");
    arena.set_scope(Some(parent));
    let child = arena.new_node_in_scope("child");

    let names: Vec<_> = child.ancestors(&arena).map(|id| *arena[id].get()).collect();
    assert_eq!(names, vec!["child", "parent"]);
}

#[test]
fn set_scope_returns_previous() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let node1 = arena.new_node_in_scope("node1");
    let node2 = arena.new_node_in_scope("node2");

    let prev = arena.set_scope(Some(node1));
    assert!(prev.is_none());

    let prev = arena.set_scope(Some(node2));
    assert_eq!(prev, Some(node1));
}

#[test]
fn nodes_in_scope_returns_children() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let parent = arena.new_node_in_scope("parent");
    arena.set_scope(Some(parent));
    arena.new_node_in_scope("child1");
    arena.new_node_in_scope("child2");

    let children: Vec<_> = arena.nodes_in_scope().map(|n| *n.get()).collect();
    assert_eq!(children, vec!["child1", "child2"]);
}

#[test]
fn nodes_in_scope_returns_roots_when_unscoped() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    arena.new_node_in_scope("root1");
    arena.new_node_in_scope("root2");

    let roots: Vec<_> = arena.nodes_in_scope().map(|n| *n.get()).collect();
    assert_eq!(roots, vec!["root1", "root2"]);
}

#[test]
fn descendants_from_scope_traverses_hierarchy() {
    let mut arena: ScopedArena<&str> = ScopedArena::new();
    let parent = arena.new_node_in_scope("parent");
    arena.set_scope(Some(parent));
    let child = arena.new_node_in_scope("child");
    arena.set_scope(Some(child));
    arena.new_node_in_scope("grandchild");

    arena.set_scope(Some(parent));
    let descendants: Vec<_> = arena.descendants_from_scope().map(|n| *n.get()).collect();
    assert_eq!(descendants, vec!["parent", "child", "grandchild"]);
}
