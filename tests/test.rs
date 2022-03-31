// use std::convert::From;
use behavior_tree_lite::{BehaviorNode, BehaviorResult, Context};

struct CheckMeNode;

impl BehaviorNode for CheckMeNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        assert_eq!(Some(&"check me"), ctx.get::<&str>("check"));
        BehaviorResult::Success
    }
}

#[test]
fn test_arm() {
    let mut ctx = Context::default();
    ctx.set("check", "check me");
    let mut print_arm = CheckMeNode;
    print_arm.tick(&mut ctx);
}
