mod error;
mod nodes;
mod parser;

use std::collections::HashMap;

pub use crate::nodes::{FallbackNode, SequenceNode};
pub use crate::parser::{load_yaml, Constructor, Registry};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
}

#[derive(Default, Debug)]
pub struct Context {
    blackboard: HashMap<String, Box<dyn std::any::Any>>,
    blackboard_map: HashMap<String, String>,
}

impl Context {
    pub fn get<'a, T: 'static>(&'a self, key: &str) -> Option<&'a T> {
        let mapped = self
            .blackboard_map
            .get(key)
            .map(|key| key as &str)
            .unwrap_or(key);
        // println!("mapped: {:?}", mapped);
        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<S: ToString, T: 'static>(&mut self, key: S, val: T) {
        let key = key.to_string();
        let mapped = self.blackboard_map.get(&key).cloned().unwrap_or(key);
        self.blackboard.insert(mapped, Box::new(val));
    }
}

pub trait BehaviorNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult;

    fn add_child(&mut self, _val: Box<dyn BehaviorNode>, _blackboard_map: HashMap<String, String>) {
    }
}

pub struct BehaviorNodeContainer {
    node: Box<dyn BehaviorNode>,
    blackboard_map: HashMap<String, String>,
}

#[macro_export]
macro_rules! hash_map {
    () => {
        std::collections::HashMap::default()
    };
    ($name: literal => $val: expr) => {{
        let mut ret = std::collections::HashMap::default();
        ret.insert($name.to_string(), $val.to_string());
        ret
    }};
}
