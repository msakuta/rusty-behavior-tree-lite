use super::*;

struct Append<const V: bool = true>;

impl<const V: bool> BehaviorNode for Append<V> {
    fn tick(&mut self, arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        arg(&V);
        BehaviorResult::Success
    }
}

#[test]
fn test_sequence() {
    let mut res = vec![];

    let mut append = |v: &dyn std::any::Any| {
        res.push(*v.downcast_ref::<bool>().unwrap());
        None
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
        tree.tick(&mut |_| None, &mut Context::default())
    );
}

struct Suspend;

impl BehaviorNode for Suspend {
    fn tick(&mut self, _arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
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
            &mut |v: &dyn std::any::Any| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut Context::default(),
        ),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Even ticking again won't invoke push(false)
    tree.tick(
        &mut |v: &dyn std::any::Any| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
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
        tree.tick(
            &mut |v: &dyn std::any::Any| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut Context::default(),
        ),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Unlike a SequenceNode, ticking again will invoke push(true) again
    tree.tick(
        &mut |v: &dyn std::any::Any| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut Context::default(),
    );

    assert_eq!(res, vec![true, true]);
}

struct AppendAndFail<const V: bool = true>;

impl<const V: bool> BehaviorNode for AppendAndFail<V> {
    fn tick(&mut self, arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        arg(&V);
        BehaviorResult::Fail
    }
}

#[test]
fn test_fallback() {
    let mut res = vec![];

    let mut append = |v: &dyn std::any::Any| {
        res.push(*v.downcast_ref::<bool>().unwrap());
        None
    };

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
        tree.tick(
            &mut |v: &dyn std::any::Any| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut Context::default(),
        ),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Even ticking again won't invoke push(false)
    tree.tick(
        &mut |v: &dyn std::any::Any| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut Context::default(),
    );

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
        tree.tick(
            &mut |v: &dyn std::any::Any| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut Context::default(),
        ),
        BehaviorResult::Running
    );

    assert_eq!(res, vec![true]);

    // Unlike a FallbackNode, ticking again will invoke push(true) again
    tree.tick(
        &mut |v: &dyn std::any::Any| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut Context::default(),
    );

    assert_eq!(res, vec![true, true]);
}

struct AlwaysSucceed;

impl BehaviorNode for AlwaysSucceed {
    fn tick(&mut self, _arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Success
    }
}

struct AlwaysFail;

impl BehaviorNode for AlwaysFail {
    fn tick(&mut self, _arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Fail
    }
}

#[test]
fn test_force_success() {
    let mut success_success = ForceSuccessNode::default();
    success_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut success_failure = ForceSuccessNode::default();
    success_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_failure.tick(&mut |_| None, &mut Context::default())
    );
}

#[test]
fn test_force_failure() {
    let mut failure_success = ForceFailureNode::default();
    failure_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut failure_failure = ForceFailureNode::default();
    failure_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_failure.tick(&mut |_| None, &mut Context::default())
    );
}

#[test]
fn test_inverter() {
    let mut invert_success = InverterNode::default();
    invert_success
        .add_child(Box::new(AlwaysSucceed), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        invert_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut invert_failure = InverterNode::default();
    invert_failure
        .add_child(Box::new(AlwaysFail), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        invert_failure.tick(&mut |_| None, &mut Context::default())
    );

    let mut invert_running = InverterNode::default();
    invert_running
        .add_child(Box::new(Suspend), BBMap::new())
        .unwrap();

    assert_eq!(
        BehaviorResult::Running,
        invert_running.tick(&mut |_| None, &mut Context::default())
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
    while let BehaviorResult::Running = tree.tick(
        &mut |v| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut ctx,
    ) {}
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
    while let BehaviorResult::Running = tree.tick(
        &mut |v| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut ctx,
    ) {}
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
    while let BehaviorResult::Running = tree.tick(
        &mut |v| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut ctx,
    ) {}
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
    while let BehaviorResult::Running = tree.tick(
        &mut |v| {
            res.push(*v.downcast_ref::<bool>().unwrap());
            None
        },
        &mut ctx,
    ) {}
    assert_eq!(res, vec![true; 3]);
}
