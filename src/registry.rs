use crate::ContextProvider;

use super::{
    nodes::{
        FallbackNode, ForceFailureNode, ForceSuccessNode, IfNode, InverterNode, IsTrueNode,
        ReactiveFallbackNode, ReactiveSequenceNode, RepeatNode, RetryNode, SequenceNode,
        SetBoolNode,
    },
    BehaviorNode, Symbol,
};
use std::collections::HashMap;

pub trait Constructor<P>: Fn() -> Box<dyn BehaviorNode<P>> {}

pub fn boxify<P, T>(cons: impl (Fn() -> T) + 'static) -> Box<dyn Fn() -> Box<dyn BehaviorNode<P>>>
where
    for<'a> T: BehaviorNode<P> + 'static,
    P: ContextProvider,
{
    Box::new(move || Box::new(cons()))
}

pub struct Registry<P> {
    node_types: HashMap<String, Box<dyn Fn() -> Box<dyn BehaviorNode<P>>>>,
    pub(crate) key_names: HashMap<String, Symbol>,
}

impl<P> Default for Registry<P>
where
    P: ContextProvider,
{
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
        ret.register("IsTrue", boxify(|| IsTrueNode));
        ret.register("if", boxify(IfNode::default));
        ret.register("SetBool", boxify(|| SetBoolNode));
        ret
    }
}

impl<P> Registry<P> {
    pub fn register(
        &mut self,
        type_name: impl ToString,
        constructor: Box<dyn Fn() -> Box<dyn BehaviorNode<P>>>,
    ) {
        self.node_types.insert(type_name.to_string(), constructor);
    }

    pub fn build(&self, type_name: &str) -> Option<Box<dyn BehaviorNode<P>>> {
        self.node_types
            .get(type_name)
            .map(|constructor| constructor())
    }
}
