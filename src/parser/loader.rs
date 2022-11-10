use std::collections::HashMap;

use super::nom_parser::{TreeDef, TreeSource};
use crate::{BehaviorNode, Registry};

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
        ret.add_child(load_recurse(child, registry)?, HashMap::new());
    }

    Ok(ret)
}
