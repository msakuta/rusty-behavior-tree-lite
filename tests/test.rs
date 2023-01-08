// use std::convert::From;
use behavior_tree_lite::{
    hash_map, BehaviorCallback, BehaviorNode, BehaviorResult, Context, ContextProvider,
    FallbackNode, SequenceNode, Symbol,
};

struct NullProvider;

impl ContextProvider for NullProvider {
    type Send = ();
    type Recv = ();
}

struct CheckMeNode;

impl<P> BehaviorNode<P> for CheckMeNode
where
    P: ContextProvider,
{
    fn tick(&mut self, _arg: BehaviorCallback<P>, ctx: &mut Context) -> BehaviorResult {
        assert_eq!(Some(&"check me"), ctx.get(Symbol::from("check")));
        BehaviorResult::Success
    }
}

#[test]
fn test_check() {
    let mut ctx = Context::default();
    ctx.set(Symbol::from("check"), "check me");
    let mut print_arm = CheckMeNode;
    <CheckMeNode as BehaviorNode<NullProvider>>::tick(&mut print_arm, &mut |_| (), &mut ctx);
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
fn test_sequence() {
    let mut seq = SequenceNode::<NullProvider>::default();
    seq.add_child(Box::new(AlwaysSucceed), hash_map!()).unwrap();
    seq.add_child(Box::new(AlwaysSucceed), hash_map!()).unwrap();
    assert_eq!(
        seq.tick(&mut |_| (), &mut Context::default()),
        BehaviorResult::Success
    );
    seq.add_child(Box::new(AlwaysFail), hash_map!()).unwrap();
    assert_eq!(
        seq.tick(&mut |_| (), &mut Context::default()),
        BehaviorResult::Fail
    );
}

#[test]
fn test_fallback() {
    let mut seq = FallbackNode::<NullProvider>::default();
    seq.add_child(Box::new(AlwaysFail), hash_map!()).unwrap();
    seq.add_child(Box::new(AlwaysFail), hash_map!()).unwrap();
    assert_eq!(
        seq.tick(&mut |_| (), &mut Context::default()),
        BehaviorResult::Fail
    );
    seq.add_child(Box::new(AlwaysSucceed), hash_map!()).unwrap();
    assert_eq!(
        seq.tick(&mut |_| (), &mut Context::default()),
        BehaviorResult::Success
    );
}
