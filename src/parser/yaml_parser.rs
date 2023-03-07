use crate::{error::LoadYamlError, BBMap, BehaviorNode, BlackboardValue, PortType, Registry};
use serde_yaml::Value;
use std::collections::HashMap;

type ParseResult = Result<Option<(Box<dyn BehaviorNode>, BBMap)>, LoadYamlError>;

fn recurse_parse(value: &serde_yaml::Value, reg: &Registry) -> ParseResult {
    let mut node = if let Some(node) =
        value
            .get("type")
            .and_then(|value| value.as_str())
            .and_then(|value| {
                eprintln!("Returning {value}");
                reg.build(value)
            }) {
        node
    } else {
        eprintln!("Type does not exist in value {value:?}");
        return Ok(None);
    };

    if let Some(Value::Sequence(children)) = value.get(&Value::from("children")) {
        for child in children {
            if let Some(built_child) = recurse_parse(child, reg)? {
                node.add_child(built_child.0, built_child.1)
                    .map_err(LoadYamlError::AddChildError)?;
            }
        }
    }

    let blackboard_map = if let Some(Value::Mapping(ports)) = value.get(&Value::from("ports")) {
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

    Ok(Some((node, blackboard_map)))
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
                            .map(|v| v.0)
                            .ok_or_else(|| LoadYamlError::Missing)?,
                    ))
                })
                .collect::<Result<_, LoadYamlError>>();
        }
    }

    Err(LoadYamlError::Missing)
}
