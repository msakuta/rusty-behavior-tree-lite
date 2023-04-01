use crate::{
    BehaviorCallback, BehaviorNode, BehaviorNodeContainer, BehaviorResult, Blackboard, Context,
    Lazy, NumChildren, PortSpec, PortType, Symbol,
};

pub fn tick_child_node<T>(
    arg: BehaviorCallback,
    ctx: &mut Context,
    node: &mut BehaviorNodeContainer,
) -> BehaviorResult {
    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
    let res = node.node.tick(arg, ctx);
    std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
    res
}

/// SubtreeNode is a container for a subtree, introducing a local namescope of blackboard variables.
pub struct SubtreeNode {
    /// Blackboard variables needs to be a part of the node payload
    blackboard: Blackboard,
    params: Vec<PortSpec>,
}

impl SubtreeNode {
    pub fn new(blackboard: Blackboard, params: Vec<PortSpec>) -> Self {
        Self { blackboard, params }
    }
}

impl BehaviorNode for SubtreeNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        self.params.clone()
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        for param in self
            .params
            .iter()
            .filter(|param| matches!(param.ty, PortType::Input | PortType::InOut))
        {
            if let Some(value) = ctx.get_any(param.key) {
                self.blackboard.insert(param.key, value.clone());
            }
        }

        std::mem::swap(&mut self.blackboard, &mut ctx.blackboard);
        let res = ctx.tick_child(0, arg);
        std::mem::swap(&mut ctx.blackboard, &mut self.blackboard);

        // It is debatable if we should assign the output value back to the parent blackboard
        // when the result was Fail or Running. We chose to assign them, which seems less counterintuitive.
        for param in self
            .params
            .iter()
            .filter(|param| matches!(param.ty, PortType::Output | PortType::InOut))
        {
            if let Some(value) = self.blackboard.get(&param.key) {
                ctx.set_any(param.key, value.clone());
            }
        }

        res.unwrap_or(BehaviorResult::Fail)
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
    }
}

#[derive(Default)]
pub struct SequenceNode {
    current_child: Option<usize>,
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let from = self.current_child.unwrap_or(0);
        for i in from..ctx.num_children() {
            match ctx.tick_child(i, arg) {
                Some(BehaviorResult::Fail) => {
                    self.current_child = None;
                    return BehaviorResult::Fail;
                }
                Some(BehaviorResult::Running) => {
                    self.current_child = Some(i);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
        }
        self.current_child = None;
        BehaviorResult::Success
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Infinite
    }
}

#[derive(Default)]
pub struct ReactiveSequenceNode;

impl BehaviorNode for ReactiveSequenceNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        for i in 0..ctx.num_children() {
            match ctx.tick_child(i, arg) {
                Some(BehaviorResult::Fail) => {
                    return BehaviorResult::Fail;
                }
                Some(BehaviorResult::Running) => {
                    return BehaviorResult::Running;
                }
                _ => (),
            }
        }
        BehaviorResult::Success
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Infinite
    }
}

#[derive(Default)]
pub struct FallbackNode {
    current_child: Option<usize>,
}

impl BehaviorNode for FallbackNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let from = self.current_child.unwrap_or(0);
        for i in from..ctx.num_children() {
            match ctx.tick_child(i, arg) {
                Some(BehaviorResult::Success) => {
                    self.current_child = None;
                    return BehaviorResult::Success;
                }
                Some(BehaviorResult::Running) => {
                    self.current_child = Some(i);
                    return BehaviorResult::Running;
                }
                _ => (),
            }
        }
        self.current_child = None;
        BehaviorResult::Fail
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Infinite
    }
}

#[derive(Default)]
pub struct ReactiveFallbackNode;

impl BehaviorNode for ReactiveFallbackNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        for i in 0..ctx.num_children() {
            match ctx.tick_child(i, arg) {
                Some(BehaviorResult::Success) => {
                    return BehaviorResult::Success;
                }
                Some(BehaviorResult::Running) => {
                    return BehaviorResult::Running;
                }
                _ => (),
            }
        }
        BehaviorResult::Fail
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Infinite
    }
}

#[derive(Default)]
pub struct ForceSuccessNode;

impl BehaviorNode for ForceSuccessNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        match ctx.tick_child(0, arg) {
            Some(BehaviorResult::Running) => BehaviorResult::Running,
            Some(_) => BehaviorResult::Success,
            _ => BehaviorResult::Fail,
        }
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
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

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
    }
}

#[derive(Default)]
pub struct InverterNode;

impl BehaviorNode for InverterNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        match ctx.tick_child(0, arg) {
            Some(BehaviorResult::Running) => BehaviorResult::Running,
            Some(BehaviorResult::Success) => BehaviorResult::Fail,
            Some(BehaviorResult::Fail) => BehaviorResult::Success,
            None => BehaviorResult::Fail,
        }
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
    }
}

static N: Lazy<Symbol> = Lazy::new(|| "n".into());

#[derive(Default)]
pub(super) struct RepeatNode {
    n: Option<usize>,
}

impl BehaviorNode for RepeatNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(current) = self.n.or_else(|| ctx.get_parse::<usize>("n")) {
            if current == 0 {
                self.n = None;
                return BehaviorResult::Success;
            }
            match ctx.tick_child(0, arg) {
                Some(BehaviorResult::Success) => {
                    self.n = Some(current - 1);
                    return BehaviorResult::Running;
                }
                Some(BehaviorResult::Running) => return BehaviorResult::Running,
                Some(res) => {
                    self.n = None;
                    return res;
                }
                _ => return BehaviorResult::Fail,
            }
        }
        BehaviorResult::Fail
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
    }
}

#[derive(Default)]
pub(super) struct RetryNode {
    n: Option<usize>,
}

impl BehaviorNode for RetryNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(current) = self.n.or_else(|| ctx.get_parse::<usize>("n")) {
            if current == 0 {
                self.n = None;
                return BehaviorResult::Success;
            }
            match ctx.tick_child(0, arg) {
                Some(BehaviorResult::Fail) => {
                    self.n = Some(current - 1);
                    return BehaviorResult::Running;
                }
                Some(BehaviorResult::Running) => return BehaviorResult::Running,
                Some(res) => {
                    self.n = None;
                    return res;
                }
                _ => return BehaviorResult::Fail,
            }
        }
        BehaviorResult::Fail
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(1)
    }
}

pub(crate) static VALUE: Lazy<Symbol> = Lazy::new(|| "value".into());
pub(crate) static OUTPUT: Lazy<Symbol> = Lazy::new(|| "output".into());

pub(crate) struct SetBoolNode;

impl BehaviorNode for SetBoolNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*VALUE), PortSpec::new_out(*OUTPUT)]
    }

    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let result = ctx.get_parse::<bool>(*VALUE);
        if let Some(value) = result {
            ctx.set(*OUTPUT, value);
            BehaviorResult::Success
        } else {
            BehaviorResult::Fail
        }
    }
}

pub(crate) static INPUT: Lazy<Symbol> = Lazy::new(|| "input".into());

pub struct IsTrueNode;

impl BehaviorNode for IsTrueNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*INPUT)]
    }

    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(input) = ctx.get_parse::<bool>(*INPUT) {
            if input {
                BehaviorResult::Success
            } else {
                BehaviorResult::Fail
            }
        } else {
            BehaviorResult::Fail
        }
    }
}

#[derive(Default)]
pub struct IfNode {
    condition_result: Option<BehaviorResult>,
}

impl BehaviorNode for IfNode {
    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let condition_result = match self.condition_result {
            Some(BehaviorResult::Running) => ctx.tick_child(0, arg).unwrap_or(BehaviorResult::Fail),
            Some(res) => res,
            None => ctx.tick_child(0, arg).unwrap_or(BehaviorResult::Fail),
        };

        // Remember the last conditional result in case the child node returns Running
        self.condition_result = Some(condition_result);

        if matches!(condition_result, BehaviorResult::Running) {
            return BehaviorResult::Running;
        }

        let branch_result = match condition_result {
            BehaviorResult::Success => ctx.tick_child(1, arg).unwrap_or(BehaviorResult::Fail),
            BehaviorResult::Fail => {
                // Be aware that lack of else clause is not an error, so the result is Success.
                ctx.tick_child(2, arg).unwrap_or(BehaviorResult::Success)
            }
            BehaviorResult::Running => BehaviorResult::Running,
        };

        // Clear the last state if either true or false branch has succeeded. This node should
        // evaluate condition again if it's ticked later.
        if !matches!(branch_result, BehaviorResult::Running) {
            self.condition_result = None;
        }

        branch_result
    }

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(3)
    }
}

#[cfg(test)]
mod test;
