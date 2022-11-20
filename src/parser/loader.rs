use super::nom_parser::{TreeDef, TreeSource};
use crate::{BBMap, BehaviorNode, Registry};

pub fn load(
    tree_source: &TreeSource,
    registry: &Registry,
) -> Result<Box<dyn BehaviorNode>, String> {
    let main = tree_source
        .tree_defs
        .iter()
        .find(|tree| tree.name == "main")
        .ok_or_else(|| "Main tree does not exist".to_string())?;

    load_recurse(&main.root, registry, tree_source)
}

fn load_recurse(
    parent: &TreeDef,
    registry: &Registry,
    tree_source: &TreeSource,
) -> Result<Box<dyn BehaviorNode>, String> {
    let mut ret = registry
        .build(parent.ty)
        .or_else(|| {
            tree_source
                .tree_defs
                .iter()
                .find(|tree| tree.name == parent.ty)
                .and_then(|tree| load_recurse(&tree.root, registry, tree_source).ok())
        })
        .ok_or_else(|| format!("Node type or subtree name not found {:?}", parent.ty))?;

    for child in &parent.children {
        let mut bbmap = BBMap::new();
        for entry in child.port_maps.iter() {
            bbmap.insert(
                entry.node_port.into(),
                match entry.blackboard_value {
                    super::nom_parser::BlackboardValue::Ref(ref value) => {
                        crate::BlackboardValue::Ref(value.into())
                    }
                    super::nom_parser::BlackboardValue::Literal(ref value) => {
                        crate::BlackboardValue::Literal(value.clone())
                    }
                },
            );
        }
        ret.add_child(load_recurse(child, registry, tree_source)?, bbmap);
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
        let mut tree = load(&tree_source, &registry).unwrap();

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
}
