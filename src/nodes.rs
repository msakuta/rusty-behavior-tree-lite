use crate::{
    BBMap, BehaviorCallback, BehaviorNode, BehaviorNodeContainer, BehaviorResult, Context,
};

pub struct SequenceNode {
    children: Vec<BehaviorNodeContainer>,
    current_child: Option<usize>,
}

impl Default for SequenceNode {
    fn default() -> Self {
        Self {
            children: vec![],
            current_child: None,
        }
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let from = self.current_child.unwrap_or(0);
        for (i, node) in self.children[from..].iter_mut().enumerate() {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            match node.node.tick(arg, ctx) {
                BehaviorResult::Fail => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Fail;
                }
                BehaviorResult::Running => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    self.current_child = Some(i + from);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
        }
        self.current_child = None;
        BehaviorResult::Success
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

pub struct ReactiveSequenceNode {
    children: Vec<BehaviorNodeContainer>,
}

impl Default for ReactiveSequenceNode {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl BehaviorNode for ReactiveSequenceNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            match node.node.tick(arg, ctx) {
                BehaviorResult::Fail => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Fail;
                }
                BehaviorResult::Running => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
        }
        BehaviorResult::Success
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

pub struct FallbackNode {
    children: Vec<BehaviorNodeContainer>,
    current_child: Option<usize>,
}

impl Default for FallbackNode {
    fn default() -> Self {
        Self {
            children: vec![],
            current_child: None,
        }
    }
}

impl BehaviorNode for FallbackNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let from = self.current_child.unwrap_or(0);
        for (i, node) in self.children[from..].iter_mut().enumerate() {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            match node.node.tick(arg, ctx) {
                BehaviorResult::Success => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Success;
                }
                BehaviorResult::Running => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    self.current_child = Some(i + from);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
        }
        self.current_child = None;
        BehaviorResult::Fail
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

pub struct ReactiveFallbackNode {
    children: Vec<BehaviorNodeContainer>,
}

impl Default for ReactiveFallbackNode {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl BehaviorNode for ReactiveFallbackNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            match node.node.tick(arg, ctx) {
                BehaviorResult::Success => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Success;
                }
                BehaviorResult::Running => {
                    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
        }
        BehaviorResult::Fail
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

#[derive(Default)]
pub struct ForceSuccessNode(Option<BehaviorNodeContainer>);

impl BehaviorNode for ForceSuccessNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(ref mut node) = self.0 {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if let BehaviorResult::Running = node.node.tick(arg, ctx) {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Running;
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            BehaviorResult::Success
        } else {
            BehaviorResult::Fail
        }
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.0 = Some(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

#[derive(Default)]
pub struct ForceFailureNode(Option<BehaviorNodeContainer>);

impl BehaviorNode for ForceFailureNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(ref mut node) = self.0 {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if let BehaviorResult::Running = node.node.tick(arg, ctx) {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Running;
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            BehaviorResult::Fail
        } else {
            BehaviorResult::Fail
        }
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.0 = Some(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

#[derive(Default)]
pub struct InverterNode(Option<BehaviorNodeContainer>);

impl BehaviorNode for InverterNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(ref mut node) = self.0 {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            let res = match node.node.tick(arg, ctx) {
                BehaviorResult::Running => BehaviorResult::Running,
                BehaviorResult::Success => BehaviorResult::Fail,
                BehaviorResult::Fail => BehaviorResult::Success,
            };
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            res
        } else {
            BehaviorResult::Fail
        }
    }

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) {
        self.0 = Some(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
    }
}

#[cfg(test)]
mod test;
