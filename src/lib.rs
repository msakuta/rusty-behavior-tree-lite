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

#[derive(Debug)]
pub enum BlackboardValue {
    Ref(Symbol),
    Literal(String),
}

pub type BBMap = HashMap<Symbol, BlackboardValue>;

#[derive(Default, Debug)]
pub struct Context<'e, T: 'e = ()> {
    blackboard: HashMap<Symbol, Box<dyn std::any::Any>>,
    blackboard_map: BBMap,
    pub env: Option<&'e mut T>,
}

impl<'e, T> Context<'e, T> {
    pub fn new(env: &'e mut T) -> Self {
        Self {
            blackboard: HashMap::new(),
            blackboard_map: HashMap::new(),
            env: Some(env),
        }
    }
}

impl<'e, E> Context<'e, E> {
    pub fn get<'a, T: 'static>(&'a self, key: Symbol) -> Option<&'a T>
    where
        'e: 'a,
    {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => &key,
            Some(BlackboardValue::Ref(mapped)) => mapped,
            Some(BlackboardValue::Literal(mapped)) => {
                return (mapped as &dyn std::any::Any).downcast_ref();
            }
        };

        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<T: 'static>(&mut self, key: Symbol, val: T) {
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

pub trait BehaviorNode {
    fn tick<'a>(
        &mut self,
        arg: &mut dyn FnMut(&dyn std::any::Any),
        ctx: &mut Context,
    ) -> BehaviorResult;

    fn add_child(&mut self, _val: Box<dyn BehaviorNode>, _blackboard_map: BBMap) {}
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
