mod context;
pub mod error;
mod nodes;
mod parser;
mod port;
mod registry;
mod symbol;

use std::any::Any;
use std::collections::HashMap;

pub use crate::context::Context;
pub use crate::nodes::{FallbackNode, SequenceNode};
pub use crate::symbol::Symbol;
pub use crate::{
    parser::{load, load_yaml, node_def, parse_file, parse_nodes, NodeDef},
    port::{PortSpec, PortType},
    registry::{boxify, Constructor, Registry},
};
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
    Ref(Symbol, PortType),
    Literal(String),
}

pub type Blackboard = HashMap<Symbol, Box<dyn Any>>;
pub type BBMap = HashMap<Symbol, BlackboardValue>;
pub type BehaviorCallback<'a> = &'a mut dyn FnMut(&dyn Any) -> Option<Box<dyn Any>>;

pub trait BehaviorNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![]
    }

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
