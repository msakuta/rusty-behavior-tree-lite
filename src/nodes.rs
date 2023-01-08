use crate::{
    error::{AddChildError, AddChildResult},
    BBMap, BehaviorCallback, BehaviorNode, BehaviorNodeContainer, BehaviorResult, Blackboard,
    Context, ContextProvider, Lazy, PortSpec, PortType, Symbol,
};

pub struct NullProvider;

impl ContextProvider for NullProvider {
    type Send = ();
    type Recv = ();
}

/// SubtreeNode is a container for a subtree, introducing a local namescope of blackboard variables.
pub struct SubtreeNode<P> {
    child: BehaviorNodeContainer<P>,
    /// Blackboard variables needs to be a part of the node payload
    blackboard: Blackboard,
    params: Vec<PortSpec>,
}

impl<P> SubtreeNode<P>
where
    P: ContextProvider,
{
    pub fn new(
        child: Box<dyn BehaviorNode<P>>,
        blackboard: Blackboard,
        params: Vec<PortSpec>,
    ) -> Self {
        Self {
            child: BehaviorNodeContainer {
                node: child,
                blackboard_map: BBMap::new(),
            },
            blackboard,
            params,
        }
    }
}

impl<P> BehaviorNode<P> for SubtreeNode<P>
where
    P: ContextProvider,
{
    fn provided_ports(&self) -> Vec<PortSpec> {
        self.params.clone()
    }

    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
        for param in self
            .params
            .iter()
            .filter(|param| matches!(param.ty, PortType::Input | PortType::InOut))
        {
            if let Some(value) = ctx.get_any(param.key) {
                self.blackboard.insert(param.key, value.clone());
            }
        }
        std::mem::swap(&mut ctx.blackboard, &mut self.blackboard);
        std::mem::swap(&mut ctx.blackboard_map, &mut self.child.blackboard_map);
        let res = self.child.node.tick(arg, ctx);
        std::mem::swap(&mut ctx.blackboard_map, &mut self.child.blackboard_map);
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

        res
    }

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        self.child = BehaviorNodeContainer {
            node,
            blackboard_map,
        };
        Ok(())
    }
}

pub struct SequenceNode<P: ContextProvider> {
    children: Vec<BehaviorNodeContainer<P>>,
    current_child: Option<usize>,
}

impl<P> Default for SequenceNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self {
            children: vec![],
            current_child: None,
        }
    }
}

impl<P> BehaviorNode<P> for SequenceNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
    }
}

pub struct ReactiveSequenceNode<P> {
    children: Vec<BehaviorNodeContainer<P>>,
}

impl<P> Default for ReactiveSequenceNode<P> {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl<P> BehaviorNode<P> for ReactiveSequenceNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
    }
}

pub struct FallbackNode<P> {
    children: Vec<BehaviorNodeContainer<P>>,
    current_child: Option<usize>,
}

impl<P> Default for FallbackNode<P> {
    fn default() -> Self {
        Self {
            children: vec![],
            current_child: None,
        }
    }
}

impl<P> BehaviorNode<P> for FallbackNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
    }
}

pub struct ReactiveFallbackNode<P> {
    children: Vec<BehaviorNodeContainer<P>>,
}

impl<P> Default for ReactiveFallbackNode<P> {
    fn default() -> Self {
        Self { children: vec![] }
    }
}

impl<P> BehaviorNode<P> for ReactiveFallbackNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        self.children.push(BehaviorNodeContainer {
            node,
            blackboard_map,
        });
        Ok(())
    }
}

pub struct ForceSuccessNode<P>(Option<BehaviorNodeContainer<P>>)
where
    P: ContextProvider;

impl<P> Default for ForceSuccessNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self(None)
    }
}

impl<P> BehaviorNode<P> for ForceSuccessNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
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

pub struct ForceFailureNode<P>(Option<BehaviorNodeContainer<P>>)
where
    P: ContextProvider;

impl<P> Default for ForceFailureNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self(None)
    }
}

impl<P> BehaviorNode<P> for ForceFailureNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
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

pub struct InverterNode<P>(Option<BehaviorNodeContainer<P>>);

impl<P> Default for InverterNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self(None)
    }
}

impl<P> BehaviorNode<P> for InverterNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        node: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
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

pub(super) struct RepeatNode<P> {
    n: Option<usize>,
    child: Option<BehaviorNodeContainer<P>>,
}

impl<P> Default for RepeatNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self {
            n: None,
            child: None,
        }
    }
}

impl<P> BehaviorNode<P> for RepeatNode<P>
where
    P: ContextProvider,
{
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        val: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
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

pub(super) struct RetryNode<P> {
    n: Option<usize>,
    child: Option<BehaviorNodeContainer<P>>,
}

impl<P: ContextProvider> Default for RetryNode<P> {
    fn default() -> Self {
        Self {
            n: None,
            child: None,
        }
    }
}

impl<P: ContextProvider> BehaviorNode<P> for RetryNode<P> {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*N)]
    }

    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

    fn add_child(
        &mut self,
        val: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
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

pub(crate) static VALUE: Lazy<Symbol> = Lazy::new(|| "value".into());
pub(crate) static OUTPUT: Lazy<Symbol> = Lazy::new(|| "output".into());

pub(crate) struct SetBoolNode;

impl<P> BehaviorNode<P> for SetBoolNode
where
    P: ContextProvider,
{
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*VALUE), PortSpec::new_out(*OUTPUT)]
    }

    fn tick(&mut self, _arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

impl<P> BehaviorNode<P> for IsTrueNode
where
    P: ContextProvider,
{
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in(*INPUT)]
    }

    fn tick(&mut self, _arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
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

pub struct IfNode<P> {
    children: Vec<BehaviorNodeContainer<P>>,
    condition_result: Option<BehaviorResult>,
}

impl<P> Default for IfNode<P>
where
    P: ContextProvider,
{
    fn default() -> Self {
        Self {
            children: vec![],
            condition_result: None,
        }
    }
}

impl<P> BehaviorNode<P> for IfNode<P>
where
    P: ContextProvider,
{
    fn tick(&mut self, arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
        let mut ticker = |node: &mut BehaviorNodeContainer<P>| {
            std::mem::swap(&mut node.blackboard_map, &mut ctx.blackboard_map);
            let res = node.node.tick(arg, ctx);
            std::mem::swap(&mut node.blackboard_map, &mut ctx.blackboard_map);
            res
        };

        let condition_result = self.condition_result.unwrap_or_else(|| {
            self.children
                .first_mut()
                .map(&mut ticker)
                .unwrap_or(BehaviorResult::Fail)
        });

        // Remember the last conditional result in case the child node returns Running
        self.condition_result = Some(condition_result);

        let branch_result = match condition_result {
            BehaviorResult::Success => self
                .children
                .get_mut(1)
                .map(&mut ticker)
                .unwrap_or(BehaviorResult::Fail),
            BehaviorResult::Fail => {
                // Be aware that lack of else clause is not an error, so the result is Success.
                self.children
                    .get_mut(2)
                    .map(&mut ticker)
                    .unwrap_or(BehaviorResult::Success)
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

    fn add_child(
        &mut self,
        val: Box<dyn BehaviorNode<P>>,
        blackboard_map: BBMap,
    ) -> AddChildResult {
        if self.children.len() < 3 {
            self.children.push(BehaviorNodeContainer {
                node: val,
                blackboard_map,
            });
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
    }
}

#[cfg(test)]
mod test;
