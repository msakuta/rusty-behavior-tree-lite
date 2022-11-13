mod loader;
mod nom_parser;

use crate::{error::Error, BBMap, BehaviorNode, BlackboardValue, FallbackNode, SequenceNode};
pub use loader::load;
pub use nom_parser::{node_def, parse_file, parse_nodes, NodeDef};
use serde_yaml::Value;
use std::collections::HashMap;
use symbol::Symbol;

pub trait Constructor: Fn() -> Box<dyn BehaviorNode> {}

pub struct Registry {
    node_types: HashMap<String, Box<dyn Fn() -> Box<dyn BehaviorNode>>>,
    key_names: HashMap<String, Symbol>,
}

fn sequence_constructor() -> Box<dyn BehaviorNode + 'static> {
    Box::new(SequenceNode::default())
}

fn fallback_constructor() -> Box<dyn BehaviorNode + 'static> {
    Box::new(FallbackNode::default())
}

impl Default for Registry {
    fn default() -> Self {
        let mut ret = Self {
            node_types: HashMap::new(),
            key_names: HashMap::new(),
        };
        ret.register("Sequence", Box::new(sequence_constructor));
        ret.register("Fallback", Box::new(fallback_constructor));
        ret
    }
}

impl Registry {
    pub fn register(
        &mut self,
        type_name: impl ToString,
        constructor: Box<dyn Fn() -> Box<dyn BehaviorNode>>,
    ) {
        self.node_types.insert(type_name.to_string(), constructor);
    }

    pub fn build(&self, type_name: &str) -> Option<Box<dyn BehaviorNode>> {
        self.node_types
            .get(type_name)
            .map(|constructor| constructor())
    }
}

fn recurse_parse(
    value: &serde_yaml::Value,
    reg: &Registry,
) -> serde_yaml::Result<Option<(Box<dyn BehaviorNode>, BBMap)>> {
    let mut node = if let Some(node) =
        value
            .get("type")
            .and_then(|value| value.as_str())
            .and_then(|value| {
                eprintln!("Returning {}", value);
                reg.build(value)
            }) {
        node
    } else {
        eprintln!("Type does not exist in value {:?}", value);
        return Ok(None);
    };

    if let Some(Value::Sequence(children)) = value.get(&Value::from("children")) {
        for child in children {
            if let Some(built_child) = recurse_parse(child, reg)? {
                node.add_child(built_child.0, built_child.1);
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
                        BlackboardValue::Ref(*reg.key_names.get(value)?),
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
) -> Result<HashMap<String, Box<dyn BehaviorNode>>, Error> {
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
            return Ok(roots
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.as_str().ok_or(Error::Missing)?.to_string(),
                        recurse_parse(value, reg)?
                            .map(|v| v.0)
                            .ok_or_else(|| Error::Missing)?,
                    ))
                })
                .collect::<Result<_, Error>>()?);
        }
    }

    Err(Error::Missing)
}
