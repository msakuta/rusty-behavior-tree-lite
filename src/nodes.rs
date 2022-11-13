use crate::{BehaviorNode, BehaviorNodeContainer, BehaviorResult, Context};
use std::collections::HashMap;
use symbol::Symbol;

pub struct SequenceNode {
    children: Vec<BehaviorNodeContainer>,
}

impl Default for SequenceNode {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(
        &mut self,
        arg: &mut dyn FnMut(&dyn std::any::Any),
        ctx: &mut Context,
    ) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(arg, ctx) == BehaviorResult::Fail {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: HashMap<Symbol, Symbol>) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

pub struct FallbackNode {
    children: Vec<BehaviorNodeContainer>,
}

impl Default for FallbackNode {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl BehaviorNode for FallbackNode {
    fn tick(
        &mut self,
        arg: &mut dyn FnMut(&dyn std::any::Any),
        ctx: &mut Context,
    ) -> BehaviorResult {
        let children = self.children.len();
        for (i, node) in self.children.iter_mut().enumerate() {
            println!("FallbackNode node child: {}/{}", i, children);
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(arg, ctx) == BehaviorResult::Success {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Success;
            }
            println!("FallbackNode node failed: {}/{children}", i);
        }
        BehaviorResult::Fail
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: HashMap<Symbol, Symbol>) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}
