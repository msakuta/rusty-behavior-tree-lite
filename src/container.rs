use std::collections::HashMap;

use crate::{
    error::{AddChildError, AddChildResult},
    BehaviorCallback, BehaviorNode, BehaviorResult, BlackboardValue, Context, NumChildren, Symbol,
};

pub struct BehaviorNodeContainer {
    pub(crate) node: Box<dyn BehaviorNode>,
    pub(crate) blackboard_map: HashMap<Symbol, BlackboardValue>,
    pub(crate) child_nodes: Vec<BehaviorNodeContainer>,
}

impl BehaviorNodeContainer {
    pub fn new(
        node: Box<dyn BehaviorNode>,
        blackboard_map: HashMap<Symbol, BlackboardValue>,
    ) -> Self {
        Self {
            node,
            blackboard_map,
            child_nodes: vec![],
        }
    }

    pub fn new_raw(node: Box<dyn BehaviorNode>) -> Self {
        Self {
            node,
            blackboard_map: HashMap::new(),
            child_nodes: vec![],
        }
    }

    pub fn new_node(node: impl BehaviorNode + 'static) -> Self {
        Self {
            node: Box::new(node),
            blackboard_map: HashMap::new(),
            child_nodes: vec![],
        }
    }

    pub fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        std::mem::swap(&mut self.child_nodes, &mut ctx.child_nodes.0);
        std::mem::swap(&mut self.blackboard_map, &mut ctx.blackboard_map);
        let res = self.node.tick(arg, ctx);
        std::mem::swap(&mut self.blackboard_map, &mut ctx.blackboard_map);
        std::mem::swap(&mut self.child_nodes, &mut ctx.child_nodes.0);
        res
    }

    pub fn add_child(&mut self, child: BehaviorNodeContainer) -> AddChildResult {
        if NumChildren::Finite(self.child_nodes.len()) < self.node.max_children() {
            self.child_nodes.push(child);
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
    }
}
