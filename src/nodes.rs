use crate::{
    error::{AddChildError, AddChildResult},
    BBMap, BehaviorCallback, BehaviorNode, BehaviorNodeContainer, BehaviorResult, Context, Lazy,
    PortSpec, Symbol,
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        if self.0.is_none() {
            self.0 = Some(BehaviorNodeContainer {
                node,
                blackboard_map,
            });
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        if self.0.is_none() {
            self.0 = Some(BehaviorNodeContainer {
                node,
                blackboard_map,
            });
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
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

    fn add_child(&mut self, node: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        if self.0.is_none() {
            self.0 = Some(BehaviorNodeContainer {
                node,
                blackboard_map,
            });
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
    }
}

const N: Lazy<Symbol> = Lazy::new(|| "n".into());

#[derive(Default)]
pub(super) struct RepeatNode {
    n: Option<usize>,
    child: Option<BehaviorNodeContainer>,
}

impl BehaviorNode for RepeatNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some((current, child)) = self
            .n
            .or_else(|| ctx.get_parse::<usize>("n"))
            .zip(self.child.as_mut())
        {
            if current == 0 {
                self.n = None;
                return BehaviorResult::Success;
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut child.blackboard_map);
            let res = child.node.tick(arg, ctx);
            std::mem::swap(&mut ctx.blackboard_map, &mut child.blackboard_map);
            if let BehaviorResult::Success = res {
                self.n = Some(current - 1);
                return BehaviorResult::Running;
            } else {
                return res;
            }
        }
        BehaviorResult::Fail
    }

    fn add_child(&mut self, val: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        if self.child.is_some() {
            return Err(AddChildError::TooManyNodes);
        }
        self.child = Some(BehaviorNodeContainer {
            node: val,
            blackboard_map,
        });
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct RetryNode {
    n: Option<usize>,
    child: Option<BehaviorNodeContainer>,
}

impl BehaviorNode for RetryNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some((current, child)) = self
            .n
            .or_else(|| ctx.get_parse::<usize>("n"))
            .zip(self.child.as_mut())
        {
            if current == 0 {
                self.n = None;
                return BehaviorResult::Success;
            }
            std::mem::swap(&mut ctx.blackboard_map, &mut child.blackboard_map);
            let res = child.node.tick(arg, ctx);
            std::mem::swap(&mut ctx.blackboard_map, &mut child.blackboard_map);
            if let BehaviorResult::Fail = res {
                self.n = Some(current - 1);
                return BehaviorResult::Running;
            } else {
                return res;
            }
        }
        BehaviorResult::Fail
    }

    fn add_child(&mut self, val: Box<dyn BehaviorNode>, blackboard_map: BBMap) -> AddChildResult {
        if self.child.is_some() {
            return Err(AddChildError::TooManyNodes);
        }
        self.child = Some(BehaviorNodeContainer {
            node: val,
            blackboard_map,
        });
        Ok(())
    }
}

#[cfg(test)]
mod test;
