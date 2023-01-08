use super::*;

struct NullProvider;

impl ContextProvider for NullProvider {
    type Send = ();
    type Recv = ();
}

struct AppendProvider;

impl ContextProvider for AppendProvider {
    type Send = bool;
    type Recv = ();
}

struct Append<const V: bool = true>;

impl<const V: bool> BehaviorNode<AppendProvider> for Append<V> {
    fn tick(
        &mut self,
        arg: BehaviorCallback<AppendProvider>,
        _ctx: &mut Context,
    ) -> BehaviorResult {
        arg(&V);
        BehaviorResult::Success
    }
}

#[test]
fn test_sequence() {
    let mut res = vec![];

    let mut append = |v: &bool| {
        res.push(*v);
    };

    let mut tree = SequenceNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Append::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        tree.tick(&mut append, &mut Context::default())
    );

    assert_eq!(res, vec![true, false]);

    let mut tree = SequenceNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(AppendAndFail::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        tree.tick(&mut |_| (), &mut Context::default())
    );
}

struct Suspend;

impl<P> BehaviorNode<P> for Suspend
where
    P: ContextProvider,
{
    fn tick(&mut self, _arg: BehaviorCallback<P>, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Running
    }
}

#[test]
fn test_sequence_suspend() {
    let mut res = vec![];

    let mut tree = SequenceNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Suspend), BBMap::new()).unwrap();
    tree.add_child(Box::new(Append::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        tree.tick(
            &mut |v: &bool| {
                res.push(*v);
            },
            &mut Context::default(),
        ),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Even ticking again won't invoke push(false)
    tree.tick(
        &mut |v: &bool| {
            res.push(*v);
        },
        &mut Context::default(),
    );

    assert_eq!(res, vec![true]);
}

#[test]
fn test_reactive_sequence_suspend() {
    let mut res = vec![];

    let mut tree = ReactiveSequenceNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Suspend), BBMap::new()).unwrap();
    tree.add_child(Box::new(Append::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        tree.tick(&mut |v: &bool| { res.push(*v) }, &mut Context::default(),),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Unlike a SequenceNode, ticking again will invoke push(true) again
    tree.tick(&mut |v: &bool| res.push(*v), &mut Context::default());

    assert_eq!(res, vec![true, true]);
}

struct AppendAndFail<const V: bool = true>;

impl<const V: bool> BehaviorNode<AppendProvider> for AppendAndFail<V> {
    fn tick(
        &mut self,
        arg: BehaviorCallback<AppendProvider>,
        _ctx: &mut Context,
    ) -> BehaviorResult {
        arg(&V);
        BehaviorResult::Fail
    }
}

#[test]
fn test_fallback() {
    let mut res = vec![];

    let mut append = |v: &bool| res.push(*v);

    let mut tree = FallbackNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(AppendAndFail::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        tree.tick(&mut append, &mut Context::default())
    );

    assert_eq!(res, vec![true, false]);

    let mut tree = SequenceNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Append::<false>), BBMap::new())
        .unwrap();
}

#[test]
fn test_fallback_suspend() {
    let mut res = vec![];

    let mut tree = FallbackNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Suspend), BBMap::new()).unwrap();
    tree.add_child(Box::new(AppendAndFail::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        tree.tick(&mut |v: &bool| { res.push(*v) }, &mut Context::default(),),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Even ticking again won't invoke push(false)
    tree.tick(&mut |v: &bool| res.push(*v), &mut Context::default());

    assert_eq!(res, vec![true]);
}

#[test]
fn test_reactive_fallback_suspend() {
    let mut res = vec![];

    let mut tree = ReactiveFallbackNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();
    tree.add_child(Box::new(Suspend), BBMap::new()).unwrap();
    tree.add_child(Box::new(AppendAndFail::<false>), BBMap::new())
        .unwrap();

    assert_eq!(
        tree.tick(&mut |v: &bool| { res.push(*v) }, &mut Context::default(),),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Unlike a FallbackNode, ticking again will invoke push(true) again
    tree.tick(&mut |v: &bool| res.push(*v), &mut Context::default());

    assert_eq!(res, vec![true, true]);
}

struct AlwaysSucceed;

impl<P> BehaviorNode<P> for AlwaysSucceed
where
    P: ContextProvider,
{
    fn tick(&mut self, _arg: BehaviorCallback<P>, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Success
    }
}

struct AlwaysFail;

impl<P> BehaviorNode<P> for AlwaysFail
where
    P: ContextProvider,
{
    fn tick(&mut self, _arg: BehaviorCallback<P>, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Fail
    }
}

#[test]
fn test_force_success() {
    let mut success_success = ForceSuccessNode::<NullProvider>::default();
    success_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_success.tick(&mut |_| (), &mut Context::default())
    );

    let mut success_failure = ForceSuccessNode::<NullProvider>::default();
    success_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_failure.tick(&mut |_| (), &mut Context::default())
    );
}

#[test]
fn test_force_failure() {
    let mut failure_success = ForceFailureNode::<NullProvider>::default();
    failure_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_success.tick(&mut |_| (), &mut Context::default())
    );

    let mut failure_failure = ForceFailureNode::<NullProvider>::default();
    failure_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_failure.tick(&mut |_| (), &mut Context::default())
    );
}

#[test]
fn test_inverter() {
    let mut invert_success = InverterNode::<NullProvider>::default();
    invert_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        invert_success.tick(&mut |_| (), &mut Context::default())
    );

    let mut invert_failure = InverterNode::<NullProvider>::default();
    invert_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        invert_failure.tick(&mut |_| (), &mut Context::default())
    );

    let mut invert_running = InverterNode::<NullProvider>::default();
    invert_running
        .add_child(Box::new(Suspend), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Running,
        invert_running.tick(&mut |_| (), &mut Context::default())
    );
}

#[test]
fn test_repeat() {
    let mut tree = RepeatNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    while let BehaviorResult::Running = tree.tick(&mut |v| res.push(*v), &mut ctx) {}
    assert_eq!(res, vec![true; 3]);
}

#[test]
fn test_repeat_fail() {
    let mut tree = RepeatNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    while let BehaviorResult::Running = tree.tick(&mut |v| res.push(*v), &mut ctx) {}
    assert_eq!(res, vec![true]);
}

#[test]
fn test_retry() {
    let mut tree = RetryNode::default();
    tree.add_child(Box::new(Append::<true>), BBMap::new())
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    while let BehaviorResult::Running = tree.tick(&mut |v| res.push(*v), &mut ctx) {}
    assert_eq!(res, vec![true]);
}

#[test]
fn test_retry_fail() {
    let mut tree = RetryNode::default();
    tree.add_child(Box::new(AppendAndFail::<true>), BBMap::new())
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    while let BehaviorResult::Running = tree.tick(&mut |v| res.push(*v), &mut ctx) {}
    assert_eq!(res, vec![true; 3]);
}
