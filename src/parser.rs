use crate::{error::Error, BehaviorNode, FallbackNode, SequenceNode};
use serde_yaml::Value;
use std::collections::HashMap;

pub trait Constructor {
    fn build(&self) -> Box<dyn BehaviorNode>;
}

pub struct Registry {
    node_types: HashMap<String, Box<dyn Constructor>>,
}

struct SequenceConstructor;

impl Constructor for SequenceConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        Box::new(SequenceNode::default())
    }
}

struct FallbackConstructor;

impl Constructor for FallbackConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        Box::new(FallbackNode::default())
    }
}

impl Default for Registry {
    fn default() -> Self {
        let mut ret = Self {
            node_types: HashMap::new(),
        };
        ret.register("Sequence", Box::new(SequenceConstructor));
        ret.register("Fallback", Box::new(FallbackConstructor));
        ret
    }
}

impl Registry {
    pub fn register(&mut self, type_name: impl ToString, constructor: Box<dyn Constructor>) {
        self.node_types.insert(type_name.to_string(), constructor);
    }

    pub fn build(&self, type_name: &str) -> Option<Box<dyn BehaviorNode>> {
        self.node_types
            .get(type_name)
            .map(|constructor| constructor.build())
    }
}

fn recurse_parse(
    value: &serde_yaml::Value,
    reg: &Registry,
) -> serde_yaml::Result<Option<(Box<dyn BehaviorNode>, HashMap<String, String>)>> {
    let mut node = if let Some(node) = value
        .get(&Value::from("type"))
        .and_then(|value| value.as_str())
        .and_then(|value| reg.build(value))
    {
        node
    } else {
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
                key.as_str()
                    .zip(value.as_str())
                    .map(|(key, value)| (key.to_string(), value.to_string()))
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
