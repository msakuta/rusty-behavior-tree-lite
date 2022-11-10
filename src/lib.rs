mod error;
mod nodes;
mod parser;

use std::collections::HashMap;
use symbol::Symbol;

pub use crate::nodes::{FallbackNode, SequenceNode};
pub use crate::parser::{
    load, load_yaml, node_def, parse_file, parse_nodes, Constructor, NodeDef, Registry,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
}

#[derive(Default, Debug)]
pub struct Context {
    blackboard: HashMap<Symbol, Box<dyn std::any::Any>>,
    blackboard_map: HashMap<Symbol, Symbol>,
}

impl Context {
    pub fn get<'a, T: 'static>(&'a self, key: Symbol) -> Option<&'a T> {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key).map(|key| key).unwrap_or(&key);
        // println!("mapped: {:?}", mapped);
        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<T: 'static>(&mut self, key: Symbol, val: T) {
        let mapped = self.blackboard_map.get(&key).cloned().unwrap_or(key);
        self.blackboard.insert(mapped, Box::new(val));
    }
}

pub trait BehaviorNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult;

    fn add_child(&mut self, _val: Box<dyn BehaviorNode>, _blackboard_map: HashMap<Symbol, Symbol>) {
    }
}

pub struct BehaviorNodeContainer {
    node: Box<dyn BehaviorNode>,
    blackboard_map: HashMap<Symbol, Symbol>,
}

#[macro_export]
macro_rules! hash_map {
    () => {
        std::collections::HashMap::default()
    };
    ($name: literal => $val: expr) => {{
        let mut ret = std::collections::HashMap::default();
        ret.insert($name.into(), $val.into());
        ret
    }};
}
