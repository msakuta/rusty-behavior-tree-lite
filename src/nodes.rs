use crate::{BehaviorNode, BehaviorNodeContainer, BehaviorResult, Context};
use std::collections::HashMap;

#[derive(Default)]
pub struct SequenceNode {
    children: Vec<BehaviorNodeContainer>,
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Fail {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: HashMap<String, String>) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

#[derive(Default)]
pub struct FallbackNode {
    children: Vec<BehaviorNodeContainer>,
}

impl BehaviorNode for FallbackNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Success {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Success;
            }
        }
        BehaviorResult::Fail
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: HashMap<String, String>) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}
