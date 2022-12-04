use std::collections::HashMap;

use super::nom_parser::{TreeDef, TreeSource};
use crate::{error::LoadError, nodes::SubtreeNode, BBMap, BehaviorNode, PortSpec, Registry};

/// Instantiate a behavior tree from a AST of a tree.
///
/// `check_ports` enables static checking of port availability before actually ticking.
/// It is useful to catch errors in a behavior tree source file, but you need to
/// implement [`crate::BehaviorNode::provided_ports`] to use it.
pub fn load(
    tree_source: &TreeSource,
    registry: &Registry,
    check_ports: bool,
) -> Result<Box<dyn BehaviorNode>, LoadError> {
    let main = tree_source
        .tree_defs
        .iter()
        .find(|tree| tree.name == "main")
        .ok_or_else(|| LoadError::MissingTree)?;

    let top = TreeStack {
        name: "main",
        parent: None,
    };

    load_recurse(&main.root, registry, tree_source, check_ports, &top)
}

/// A mechanism to detect infinite recursion. It is a linked list in call stack.
/// You can traverse the link back to enumerate all the subtree names (which is effectively function names)
/// and check if a subtree name to be inserted is already there.
///
/// We could also use HashSet of subtree names, but it feels silly to use dynamically allocated collection
/// when you can do the same thing with just the call stack.
///
/// # Discussion
///
/// It is very interesting discussion if we should allow recursive subtrees.
/// It will give the source file the power to describe some advanced algorithms and make it
/// easier to write Turing complete code.
///
/// However, in order to do so, we need to "lazily" load the subtree, which means we cannot instantiate
/// the behavior nodes until the subtree is actually ticked. So we need to keep [`Registry`] and [`TreeSource`]
/// objects during the lifetime of the entire behavior tree.
/// It would completely change the design of `TreeSource` and I'm not exactly sure if it's worth it.
/// After all, BehaviorTreeCPP works without recursive subtrees just fine.
/// You can always transform algorithms with recursive calls into a flat loop with an explicit stack.
///
/// Also, it is not entirely clear how we should render the behavior tree on a graphical editor, when
/// we get to implement one.
/// Clearly, we cannot expand all the subtrees that contains itself, but the user would want to expand
/// all subtrees to get better understanding of the tree structure.
/// It means the graphical editor also needs some kind of lazy evaluation.
///
/// For now, we make recursive subtrees an error. Luckily we can detect it relatively easily.
///
/// By the way, if we didn't have this mechanism in place, recursive subtrees cause a stack overflow.
/// It uses quite some amount of heap memory, but call stack runs short sooner.
struct TreeStack<'a, 'src> {
    name: &'src str,
    parent: Option<&'a TreeStack<'a, 'src>>,
}

impl<'a, 'src> TreeStack<'a, 'src> {
    fn find(&self, name: &str) -> bool {
        if self.name == name {
            true
        } else if let Some(parent) = self.parent {
            parent.find(name)
        } else {
            false
        }
    }
}

fn load_recurse(
    parent: &TreeDef,
    registry: &Registry,
    tree_source: &TreeSource,
    check_ports: bool,
    parent_stack: &TreeStack,
) -> Result<Box<dyn BehaviorNode>, LoadError> {
    let mut ret = if let Some(ret) = registry.build(parent.ty) {
        ret
    } else {
        let tree = tree_source
            .tree_defs
            .iter()
            .find(|tree| tree.name == parent.ty)
            .ok_or_else(|| LoadError::MissingNode(parent.ty.to_owned()))?;

        // Prevent infinite recursion
        if parent_stack.find(parent.ty) {
            return Err(LoadError::InfiniteRecursion {
                node: parent.ty.to_owned(),
            });
        }
        let tree_stack = TreeStack {
            name: parent.ty,
            parent: Some(parent_stack),
        };
        let loaded_subtree =
            load_recurse(&tree.root, registry, tree_source, check_ports, &tree_stack)?;
        Box::new(SubtreeNode::new(
            loaded_subtree,
            HashMap::new(),
            tree.ports
                .iter()
                .map(|port| PortSpec {
                    key: port.name.into(),
                    ty: port.direction,
                })
                .collect(),
        ))
    };

    for child in &parent.children {
        let child_node = load_recurse(child, registry, tree_source, check_ports, parent_stack)?;
        let provided_ports = child_node.provided_ports();
        let mut bbmap = BBMap::new();
        for entry in child.port_maps.iter() {
            if check_ports {
                if let Some(port) = provided_ports.iter().find(|p| p.key == entry.node_port) {
                    if port.ty != entry.ty {
                        return Err(LoadError::PortIOUnmatch {
                            node: child.ty.to_owned(),
                            port: entry.node_port.to_owned(),
                        });
                    }
                } else {
                    return Err(LoadError::PortUnmatch {
                        node: child.ty.to_owned(),
                        port: entry.node_port.to_owned(),
                    });
                }
            }
            bbmap.insert(
                entry.node_port.into(),
                match entry.blackboard_value {
                    super::nom_parser::BlackboardValue::Ref(ref value) => {
                        crate::BlackboardValue::Ref(value.into(), entry.ty)
                    }
                    super::nom_parser::BlackboardValue::Literal(ref value) => {
                        crate::BlackboardValue::Literal(value.clone())
                    }
                },
            );
        }
        ret.add_child(child_node, bbmap)
            .map_err(|e| LoadError::AddChildError(e, parent.ty.to_string()))?;
    }

    Ok(ret)
}

#[cfg(test)]
mod test;
