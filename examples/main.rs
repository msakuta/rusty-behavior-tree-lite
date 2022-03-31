use ::behavior_tree_lite::{BehaviorNode, BehaviorResult, Context, SequenceNode};
use std::any::TypeId;

struct Arm {
    name: String,
}

struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

struct PrintArmNode;

impl BehaviorNode for PrintArmNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        let id = TypeId::of::<Arm>();
        println!("Arm {:?}", id);

        if let Some(arm) = ctx.get::<String>("left") {
            println!("Got {}", arm);
        }
        BehaviorResult::Success
    }
}

struct PrintBodyNode;

impl BehaviorNode for PrintBodyNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        println!("Body");
        if let Some(body) = ctx.get::<String>("body").cloned() {
            ctx.set("left".to_string(), format!("{}'s left_arm", body));
            ctx.set("right".to_string(), format!("{}'s left_arm", body));
        }
        BehaviorResult::Success
    }
}

fn main(){
    let mut ctx = Context::default();

    let mut root = SequenceNode::default();

    root.add_child(PrintBodyNode);

    let mut print_arms = SequenceNode::default();
    print_arms.add_child(PrintArmNode);

    print_arms.tick(&mut ctx);

    root.add_child(print_arms);
}