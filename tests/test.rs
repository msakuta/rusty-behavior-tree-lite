// use std::convert::From;
use behavior_tree_lite::{hash_map, BehaviorNode, BehaviorResult, Context, SequenceNode};

struct CheckMeNode;

impl BehaviorNode for CheckMeNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        assert_eq!(Some(&"check me"), ctx.get::<&str>("check"));
        BehaviorResult::Success
    }
}

#[test]
fn test_check() {
    let mut ctx = Context::default();
    ctx.set("check", "check me");
    let mut print_arm = CheckMeNode;
    print_arm.tick(&mut ctx);
}

struct AlwaysSucceed;

impl BehaviorNode for AlwaysSucceed {
    fn tick(&mut self, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Success
    }
}

struct AlwaysFail;

impl BehaviorNode for AlwaysFail {
    fn tick(&mut self, _ctx: &mut Context) -> BehaviorResult {
        BehaviorResult::Fail
    }
}

#[test]
fn test_sequence() {
    let mut seq = SequenceNode::default();
    seq.add_child(AlwaysSucceed, hash_map!());
    seq.add_child(AlwaysSucceed, hash_map!());
    assert_eq!(seq.tick(&mut Context::default()), BehaviorResult::Success);
    seq.add_child(AlwaysFail, hash_map!());
    assert_eq!(seq.tick(&mut Context::default()), BehaviorResult::Fail);
}
