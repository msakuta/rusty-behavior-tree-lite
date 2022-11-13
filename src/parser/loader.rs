use std::collections::HashMap;

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

    load_recurse(&main.root, registry)
}

fn load_recurse(parent: &TreeDef, registry: &Registry) -> Result<Box<dyn BehaviorNode>, String> {
    let mut ret = registry
        .build(parent.ty)
        .ok_or_else(|| format!("Type not found {:?}", parent.ty))?;

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
        ret.add_child(load_recurse(child, registry)?, bbmap);
    }

    Ok(ret)
}
