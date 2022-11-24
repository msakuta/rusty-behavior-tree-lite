mod loader;
mod nom_parser;

use crate::{
    error::LoadYamlError,
    nodes::{
        ForceFailureNode, ForceSuccessNode, InverterNode, ReactiveFallbackNode,
        ReactiveSequenceNode, RepeatNode, RetryNode,
    },
    symbol::Symbol,
    BBMap, BehaviorNode, BlackboardValue, FallbackNode, SequenceNode,
};
pub use loader::load;
pub use nom_parser::{node_def, parse_file, parse_nodes, NodeDef};
use serde_yaml::Value;
use std::collections::HashMap;

pub trait Constructor: Fn() -> Box<dyn BehaviorNode> {}

pub fn boxify<T>(cons: impl (Fn() -> T) + 'static) -> Box<dyn Fn() -> Box<dyn BehaviorNode>>
where
    for<'a> T: BehaviorNode + 'static,
{
    Box::new(move || Box::new(cons()))
}

pub struct Registry {
    node_types: HashMap<String, Box<dyn Fn() -> Box<dyn BehaviorNode>>>,
    key_names: HashMap<String, Symbol>,
}

impl Default for Registry {
    fn default() -> Self {
        let mut ret = Self {
            node_types: HashMap::new(),
            key_names: HashMap::new(),
        };
        ret.register("Sequence", boxify(|| SequenceNode::default()));
        ret.register(
            "ReactiveSequence",
            boxify(|| ReactiveSequenceNode::default()),
        );
        ret.register("Fallback", boxify(|| FallbackNode::default()));
        ret.register(
            "ReactiveFallback",
            boxify(|| ReactiveFallbackNode::default()),
        );
        ret.register("ForceSuccess", boxify(|| ForceSuccessNode::default()));
        ret.register("ForceFailure", boxify(|| ForceFailureNode::default()));
        ret.register("Inverter", boxify(|| InverterNode::default()));
        ret.register("Repeat", boxify(|| RepeatNode::default()));
        ret.register("Retry", boxify(|| RetryNode::default()));
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
) -> Result<Option<(Box<dyn BehaviorNode>, BBMap)>, LoadYamlError> {
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
                node.add_child(built_child.0, built_child.1)
                    .map_err(|e| LoadYamlError::AddChildError(e))?;
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
            return Ok(roots
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.as_str().ok_or(LoadYamlError::Missing)?.to_string(),
                        recurse_parse(value, reg)?
                            .map(|v| v.0)
                            .ok_or_else(|| LoadYamlError::Missing)?,
                    ))
                })
                .collect::<Result<_, LoadYamlError>>()?);
        }
    }

    Err(LoadYamlError::Missing)
}
