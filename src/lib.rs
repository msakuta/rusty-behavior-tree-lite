mod context;
pub mod error;
mod nodes;
mod parser;
mod port;
mod registry;
mod symbol;

use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

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

/// Blackboard is a mapping of a variable names and their values.
/// The value is wrapped in an `Any` trait object, so it can be any type.
///
/// # Implementation note
///
/// You may wonder why the value is wrapped in an `Rc`, not a `Box`.
/// It seems unnecessary to have reference count for owned values inside a
/// blackboard.
/// The reason is that we need to pass the copy of the variables to the subtree by
/// copying values, but `Clone` trait is not object safe.
///
/// Another way to work around this is to define a new trait, like `AnyClone`,
/// that can clone and be object safe at the same time.
/// The signature for the clone method would be something like:
///
/// ```
/// # use std::any::Any;
/// trait AnyClone: Any {
///     fn any_clone(&self) -> Box<dyn AnyClone>;
/// }
/// ```
///
/// The difference from `Clone` trait is that it returns a boxed copy of the object
/// to avoid the requirement that returned value of a function needs to be `Sized`.
/// However, requiring every type that can exist in the blackboard to implement this
/// trait is a bit too much to ask to the users.
/// By wrapping in an `Rc`, it looks like cloneable, but it doesn't require the value
/// type to implement anything but `Any`.
///
/// Interestingly, this is the exactly the same issue when you try to implement
/// a function call in your own programming language, i.e. pass-by-value v.s. pass-by-reference.
/// In essence, a subtree in behavior tree is a function in a programming language.
/// The third sect in the society is copy-on-write reference, which is what `Rc` does.
pub type Blackboard = HashMap<Symbol, Rc<dyn Any>>;
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
