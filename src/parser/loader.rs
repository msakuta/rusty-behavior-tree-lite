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

    load_recurse(&main.root, registry, tree_source, check_ports)
}

fn load_recurse(
    parent: &TreeDef,
    registry: &Registry,
    tree_source: &TreeSource,
    check_ports: bool,
) -> Result<Box<dyn BehaviorNode>, LoadError> {
    let mut ret = if let Some(ret) = registry.build(parent.ty) {
        ret
    } else {
        let tree = tree_source
            .tree_defs
            .iter()
            .find(|tree| tree.name == parent.ty)
            .ok_or_else(|| LoadError::MissingNode(parent.ty.to_owned()))?;
        let loaded_subtree = load_recurse(&tree.root, registry, tree_source, check_ports)?;
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
        let child_node = load_recurse(child, registry, tree_source, check_ports)?;
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
mod test {
    use super::*;
    use crate::{boxify, BehaviorResult, Context};

    struct PrintNode;

    impl BehaviorNode for PrintNode {
        fn tick(
            &mut self,
            arg: crate::BehaviorCallback,
            _ctx: &mut crate::Context,
        ) -> crate::BehaviorResult {
            arg(&42);
            BehaviorResult::Success
        }
    }

    #[test]
    fn test_subtree() {
        let tree = r#"
    tree main = Sequence {
        sub
    }

    tree sub = Fallback {
        PrintNode
    }
        "#;

        let (_, tree_source) = crate::parse_file(tree).unwrap();
        let mut registry = Registry::default();
        registry.register("PrintNode", boxify(|| PrintNode));
        let mut tree = load(&tree_source, &registry, true).unwrap();

        let mut values = vec![];
        let result = tree.tick(
            &mut |val| {
                val.downcast_ref::<i32>().map(|val| values.push(*val));
                None
            },
            &mut Context::default(),
        );
        assert_eq!(result, BehaviorResult::Success);
        assert_eq!(values, vec![42]);
    }

    struct SendToArg;

    impl BehaviorNode for SendToArg {
        fn provided_ports(&self) -> Vec<PortSpec> {
            vec![PortSpec::new_in("input")]
        }

        fn tick(&mut self, arg: crate::BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
            let input = ctx.get_parse::<i32>("input").unwrap();
            arg(&input);
            BehaviorResult::Success
        }
    }

    #[test]
    fn test_subtree_map() {
        let tree = r#"
tree main = Sequence {
    sub(input <- "96")
}

tree sub(in input, out output) = Fallback {
    SendToArg (input <- input)
}
"#;
        let (_, tree_source) = crate::parse_file(tree).unwrap();
        let mut registry = Registry::default();
        registry.register("SendToArg", boxify(|| SendToArg));
        let mut tree = load(&tree_source, &registry, true).unwrap();

        let mut values = vec![];
        let result = tree.tick(
            &mut |val| {
                val.downcast_ref::<i32>().map(|val| values.push(*val));
                None
            },
            &mut Context::default(),
        );
        assert_eq!(result, BehaviorResult::Success);
        assert_eq!(values, vec![96]);
    }
}
