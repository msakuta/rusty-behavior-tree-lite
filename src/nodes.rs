use crate::{BehaviorNode, BehaviorNodeContainer, BehaviorResult, Context};
use std::collections::HashMap;
use symbol::Symbol;

pub struct SequenceNode<E> {
    children: Vec<BehaviorNodeContainer<E>>,
}

impl<E> Default for SequenceNode<E> {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl<E> BehaviorNode<E> for SequenceNode<E> {
    fn tick(&mut self, ctx: &mut Context<E>) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Fail {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<E>>,
        blackboard_map: HashMap<Symbol, Symbol>,
    ) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

pub struct FallbackNode<E> {
    children: Vec<BehaviorNodeContainer<E>>,
}

impl<E> Default for FallbackNode<E> {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl<E> BehaviorNode<E> for FallbackNode<E> {
    fn tick(&mut self, ctx: &mut Context<E>) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Success {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Success;
            }
        }
        BehaviorResult::Fail
    }

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<E>>,
        blackboard_map: HashMap<Symbol, Symbol>,
    ) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}
