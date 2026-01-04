use crate::{
    error::{AddChildError, LoadYamlError},
    BehaviorNode, BehaviorNodeContainer, BlackboardValue, NumChildren, PortType, Registry,
};
use serde_yaml::Value;
use std::collections::HashMap;

type ParseResult = Result<Option<BehaviorNodeContainer>, LoadYamlError>;

fn recurse_parse(value: &serde_yaml::Value, reg: &Registry) -> ParseResult {
    let Some(name) = value.get("type").and_then(|value| value.as_str()) else {
        return Ok(None);
    };

    let node = if let Some(node) = {
        eprintln!("Returning {name}");
        reg.build(name)
    } {
        node
    } else {
        eprintln!("Type does not exist in value {value:?}");
        return Ok(None);
    };

    let mut child_nodes = vec![];
    if let Some(Value::Sequence(children)) = value.get(Value::from("children")) {
        for child in children {
            if let Some(built_child) = recurse_parse(child, reg)? {
                if NumChildren::Finite(child_nodes.len()) < node.max_children() {
                    child_nodes.push(built_child)
                } else {
                    return Err(LoadYamlError::AddChildError(AddChildError::TooManyNodes));
                }
            }
        }
    }

    let blackboard_map = if let Some(Value::Mapping(ports)) = value.get(Value::from("ports")) {
        ports
            .iter()
            .filter_map(|(key, value)| {
                key.as_str().zip(value.as_str()).and_then(|(key, value)| {
                    Some((
                        *reg.key_names.get(key)?,
                        BlackboardValue::Ref(*reg.key_names.get(value)?, PortType::InOut),
                    ))
                })
            })
            .collect()
    } else {
        HashMap::new()
    };

    Ok(Some(BehaviorNodeContainer {
        name: name.to_owned(),
        node,
        blackboard_map,
        child_nodes,
        last_result: None,
        is_subtree: false,
        subtree_expanded: std::cell::Cell::new(false),
    }))
}

pub fn load_yaml(
    yaml: &str,
    reg: &Registry,
) -> Result<HashMap<String, Box<dyn BehaviorNode>>, LoadYamlError> {
    let yaml = serde_yaml::from_str(yaml)?;
    if let Value::Mapping(root) = yaml {
        // if let Some(Value::Mapping(nodes)) = root.get(&Value::from("nodes")) {
        //     for (key, value) in nodes {
        //         if let Some(key) = key.as_str() {
        //             println!("Item: {}", key);
        //         }
        //     }
        // }

        if let Some(Value::Mapping(roots)) = root.get(&Value::from("behavior_tree")) {
            return roots
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.as_str().ok_or(LoadYamlError::Missing)?.to_string(),
                        recurse_parse(value, reg)?
                            .map(|v| v.node)
                            .ok_or(LoadYamlError::Missing)?,
                    ))
                })
                .collect::<Result<_, LoadYamlError>>();
        }
    }

    Err(LoadYamlError::Missing)
}
