pub mod error;
mod nodes;
mod parser;
mod symbol;

use std::any::Any;
use std::collections::HashMap;

pub use crate::nodes::{FallbackNode, SequenceNode};
pub use crate::parser::{
    boxify, load, load_yaml, node_def, parse_file, parse_nodes, Constructor, NodeDef, Registry,
};
pub use crate::symbol::Symbol;
pub use ::once_cell::sync::*;
use error::{AddChildError, AddChildResult};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
    /// The node should keep running in the next tick
    Running,
}

#[derive(Debug)]
pub enum BlackboardValue {
    Ref(Symbol),
    Literal(String),
}

pub type Blackboard = HashMap<Symbol, Box<dyn Any>>;
pub type BBMap = HashMap<Symbol, BlackboardValue>;

#[derive(Default, Debug)]
pub struct Context<'e, T: 'e = ()> {
    blackboard: Blackboard,
    blackboard_map: BBMap,
    pub env: Option<&'e mut T>,
}

impl<'e, T> Context<'e, T> {
    pub fn new(blackboard: Blackboard) -> Self {
        Self {
            blackboard,
            blackboard_map: BBMap::new(),
            env: None,
        }
    }

    pub fn take_blackboard(self) -> Blackboard {
        self.blackboard
    }
}

impl<'e, E> Context<'e, E> {
    pub fn get<'a, T: 'static>(&'a self, key: impl Into<Symbol>) -> Option<&'a T>
    where
        'e: 'a,
    {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => &key,
            Some(BlackboardValue::Ref(mapped)) => mapped,
            Some(BlackboardValue::Literal(mapped)) => {
                return (mapped as &dyn Any).downcast_ref();
            }
        };

        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<T: 'static>(&mut self, key: impl Into<Symbol>, val: T) {
        let key = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => key,
            Some(BlackboardValue::Ref(mapped)) => *mapped,
            Some(BlackboardValue::Literal(_)) => panic!("Cannot write to a literal!"),
        };
        self.blackboard.insert(mapped, Box::new(val));
    }

    // pub fn get_env(&mut self) -> Option<&mut E> {
    //     self.env
    // }
}

pub type BehaviorCallback<'a> = &'a mut dyn FnMut(&dyn Any) -> Option<Box<dyn Any>>;

pub trait BehaviorNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult;

    fn add_child(&mut self, _val: Box<dyn BehaviorNode>, _blackboard_map: BBMap) -> AddChildResult {
        Err(AddChildError::TooManyNodes)
    }
}

pub struct BehaviorNodeContainer {
    node: Box<dyn BehaviorNode>,
    blackboard_map: HashMap<Symbol, BlackboardValue>,
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
