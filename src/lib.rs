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
pub struct Context<'e, E = ()> {
    blackboard: HashMap<Symbol, Box<dyn std::any::Any>>,
    blackboard_map: HashMap<Symbol, Symbol>,
    pub env: Option<&'e mut E>,
}

impl<'e, E> Context<'e, E> {
    pub fn new(env: &'e mut E) -> Self {
        Self {
            blackboard: HashMap::new(),
            blackboard_map: HashMap::new(),
            env: Some(env),
        }
    }
}

impl<'e, E> Context<'e, E> {
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

    // pub fn get_env(&mut self) -> Option<&mut E> {
    //     self.env
    // }
}

pub trait BehaviorNode<E = ()> {
    fn tick(&mut self, ctx: &mut Context<E>) -> BehaviorResult;

    fn add_child(
        &mut self,
        _val: Box<dyn BehaviorNode<E>>,
        _blackboard_map: HashMap<Symbol, Symbol>,
    ) {
    }
}

pub struct BehaviorNodeContainer<E> {
    node: Box<dyn BehaviorNode<E>>,
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
