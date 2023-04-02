use super::*;
type BNContainer = BehaviorNodeContainer;

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

    let mut tree = BNContainer::new_node(SequenceNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Append::<false>))
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        tree.tick(&mut append, &mut Context::default())
    );

    assert_eq!(res, vec![true, false]);

    let mut tree = BNContainer::new_node(SequenceNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<false>))
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

    let mut tree = BNContainer::new_node(SequenceNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Suspend)).unwrap();
    tree.add_child(BNContainer::new_node(Append::<false>))
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

    let mut tree = BNContainer::new_node(ReactiveSequenceNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Suspend)).unwrap();
    tree.add_child(BNContainer::new_node(Append::<false>))
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

    let mut tree = BNContainer::new_node(FallbackNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<false>))
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        tree.tick(&mut append, &mut Context::default())
    );

    assert_eq!(res, vec![true, false]);

    let mut tree = BNContainer::new_node(SequenceNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Append::<false>))
        .unwrap();
}

#[test]
fn test_fallback_suspend() {
    let mut res = vec![];

    let mut tree = BNContainer::new_node(FallbackNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Suspend)).unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<false>))
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

    let mut tree = BNContainer::new_node(ReactiveFallbackNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Suspend)).unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<false>))
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
    let mut success_success = BNContainer::new_node(ForceSuccessNode::default());
    success_success
        .add_child(BNContainer::new_node(AlwaysSucceed))
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut success_failure = BNContainer::new_node(ForceSuccessNode::default());
    success_failure
        .add_child(BNContainer::new_node(AlwaysFail))
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        success_failure.tick(&mut |_| None, &mut Context::default())
    );
}

#[test]
fn test_force_failure() {
    let mut failure_success = BNContainer::new_node(ForceFailureNode::default());
    failure_success
        .add_child(BNContainer::new_node(AlwaysSucceed))
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut failure_failure = BNContainer::new_node(ForceFailureNode::default());
    failure_failure
        .add_child(BNContainer::new_node(AlwaysFail))
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        failure_failure.tick(&mut |_| None, &mut Context::default())
    );
}

#[test]
fn test_inverter() {
    let mut invert_success = BNContainer::new_node(InverterNode::default());
    invert_success
        .add_child(BNContainer::new_node(AlwaysSucceed))
        .unwrap();

    assert_eq!(
        BehaviorResult::Fail,
        invert_success.tick(&mut |_| None, &mut Context::default())
    );

    let mut invert_failure = BNContainer::new_node(InverterNode::default());
    invert_failure
        .add_child(BNContainer::new_node(AlwaysFail))
        .unwrap();

    assert_eq!(
        BehaviorResult::Success,
        invert_failure.tick(&mut |_| None, &mut Context::default())
    );

    let mut invert_running = BNContainer::new_node(InverterNode::default());
    invert_running
        .add_child(BNContainer::new_node(Suspend))
        .unwrap();

    assert_eq!(
        BehaviorResult::Running,
        invert_running.tick(&mut |_| None, &mut Context::default())
    );
}

#[test]
fn test_repeat() {
    let mut tree = BNContainer::new_node(RepeatNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
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
    let mut tree = BNContainer::new_node(RepeatNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
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

struct Countdown<const C: usize>(usize);

impl<const C: usize> BehaviorNode for Countdown<C> {
    fn tick(&mut self, _arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        if self.0 == 0 {
            self.0 = C;
            BehaviorResult::Success
        } else {
            self.0 -= 1;
            BehaviorResult::Fail
        }
    }
}

impl<const C: usize> std::ops::Not for Countdown<C> {
    type Output = BNContainer;

    fn not(self) -> Self::Output {
        let mut not = BNContainer::new_node(InverterNode::default());
        not.add_child(BNContainer::new_node(self)).unwrap();
        not
    }
}

#[test]
fn test_repeat_break() {
    let mut tree = BNContainer::new_node(FallbackNode::default());
    let mut repeat = BNContainer::new_node(RepeatNode::default());
    repeat.add_child(!Countdown::<2>(2)).unwrap();
    tree.add_child(repeat).unwrap();
    tree.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    use BehaviorResult::*;

    let mut res = vec![];
    for expected in [Running, Running, Success, Running, Running, Success] {
        assert_eq!(
            tree.tick(
                &mut |v| {
                    res.push(*v.downcast_ref::<bool>().unwrap());
                    None
                },
                &mut ctx,
            ),
            expected
        );
    }
    assert_eq!(res, vec![true; 2]);
}

#[test]
fn test_repeat_suspend() {
    let mut tree = BNContainer::new_node(RepeatNode::default());
    let mut seq = BNContainer::new_node(SequenceNode::default());
    seq.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    seq.add_child(BNContainer::new_node(AlwaysRunning)).unwrap();
    tree.add_child(seq).unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    for _ in 0..3 {
        assert_eq!(
            tree.tick(
                &mut |v| {
                    res.push(*v.downcast_ref::<bool>().unwrap());
                    None
                },
                &mut ctx,
            ),
            BehaviorResult::Running
        );
    }
    // Although we repeat 3 times, the result should contain only 1 `true`, since it is suspended.
    assert_eq!(res, vec![true]);
}

#[test]
fn test_retry() {
    let mut tree = BNContainer::new_node(RetryNode::default());
    tree.add_child(BNContainer::new_node(Append::<true>))
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
    let mut tree = BNContainer::new_node(RetryNode::default());
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
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
fn test_retry_break() {
    let mut tree = BNContainer::new_node(SequenceNode::default());
    let mut retry = BNContainer::new_node(RetryNode::default());
    retry
        .add_child(BNContainer::new_node(Countdown::<2>(2)))
        .unwrap();
    tree.add_child(retry).unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    use BehaviorResult::*;

    let mut res = vec![];
    for expected in [Running, Running, Fail, Running, Running, Fail] {
        assert_eq!(
            tree.tick(
                &mut |v| {
                    res.push(*v.downcast_ref::<bool>().unwrap());
                    None
                },
                &mut ctx,
            ),
            expected
        );
    }
    assert_eq!(res, vec![true; 2]);
}

#[test]
fn test_retry_suspend() {
    let mut tree = BNContainer::new_node(RetryNode::default());
    let mut seq = BNContainer::new_node(SequenceNode::default());
    seq.add_child(BNContainer::new_node(Append::<true>))
        .unwrap();
    seq.add_child(BNContainer::new_node(AlwaysRunning)).unwrap();
    tree.add_child(seq).unwrap();

    let mut ctx = Context::default();
    ctx.set::<usize>("n", 3);

    let mut res = vec![];
    for _ in 0..3 {
        assert_eq!(
            tree.tick(
                &mut |v| {
                    res.push(*v.downcast_ref::<bool>().unwrap());
                    None
                },
                &mut ctx,
            ),
            BehaviorResult::Running
        );
    }
    // Although we repeat 3 times, the result should contain only 1 `true`, since it is suspended.
    assert_eq!(res, vec![true]);
}

#[test]
fn test_if_node() {
    let mut tree = BNContainer::new_node(IfNode::default());
    tree.add_child(BNContainer::new_node(AlwaysSucceed))
        .unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();

    let mut ctx = Context::default();

    let mut res = vec![];
    assert_eq!(
        tree.tick(
            &mut |v| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut ctx,
        ),
        BehaviorResult::Fail
    );
    assert_eq!(res, vec![true; 1]);
}

#[test]
fn test_if_node_fail() {
    let mut tree = BNContainer::new_node(IfNode::default());
    tree.add_child(BNContainer::new_node(AlwaysFail)).unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(Append::<false>))
        .unwrap();

    let mut ctx = Context::default();

    let mut res = vec![];
    assert_eq!(
        tree.tick(
            &mut |v| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut ctx,
        ),
        BehaviorResult::Success
    );
    assert_eq!(res, vec![false; 1]);
}

struct AlwaysRunning;

impl BehaviorNode for AlwaysRunning {
    fn tick(&mut self, _arg: BehaviorCallback, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Running
    }
}

#[test]
fn test_if_node_suspend() {
    let mut tree = BNContainer::new_node(IfNode::default());
    tree.add_child(BNContainer::new_node(AlwaysRunning))
        .unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();

    let mut ctx = Context::default();

    let mut res = vec![];
    assert_eq!(
        tree.tick(
            &mut |v| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut ctx,
        ),
        BehaviorResult::Running
    );
    assert_eq!(res, vec![]);
}

#[test]
fn test_if_node_true_suspend() {
    let mut tree = BNContainer::new_node(IfNode::default());
    tree.add_child(BNContainer::new_node(AlwaysSucceed))
        .unwrap();
    tree.add_child(BNContainer::new_node(AlwaysRunning))
        .unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();

    let mut ctx = Context::default();

    let mut res = vec![];
    assert_eq!(
        tree.tick(
            &mut |v| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut ctx,
        ),
        BehaviorResult::Running
    );
    assert_eq!(res, vec![]);
}

#[test]
fn test_if_node_false_suspend() {
    let mut tree = BNContainer::new_node(IfNode::default());
    tree.add_child(BNContainer::new_node(AlwaysFail)).unwrap();
    tree.add_child(BNContainer::new_node(AppendAndFail::<true>))
        .unwrap();
    tree.add_child(BNContainer::new_node(AlwaysRunning))
        .unwrap();

    let mut ctx = Context::default();

    let mut res = vec![];
    assert_eq!(
        tree.tick(
            &mut |v| {
                res.push(*v.downcast_ref::<bool>().unwrap());
                None
            },
            &mut ctx,
        ),
        BehaviorResult::Running
    );
    assert_eq!(res, vec![]);
}
