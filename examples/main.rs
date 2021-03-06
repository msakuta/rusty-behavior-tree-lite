use ::behavior_tree_lite::{hash_map, BehaviorNode, BehaviorResult, Context, SequenceNode};

#[derive(Clone, Debug)]
struct Arm {
    name: String,
}

#[derive(Debug)]
struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

struct PrintArmNode;

impl BehaviorNode for PrintArmNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        println!("Arm {:?}", ctx);

        if let Some(arm) = ctx.get::<Arm>("arm") {
            println!("Got {}", arm.name);
        }
        BehaviorResult::Success
    }
}

struct PrintBodyNode;

impl BehaviorNode for PrintBodyNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        if let Some(body) = ctx.get::<Body>("body") {
            let left_arm = body.left_arm.clone();
            let right_arm = body.right_arm.clone();
            println!("Got Body: {:?}", body);
            ctx.set("left_arm", left_arm);
            ctx.set("right_arm", right_arm);
            BehaviorResult::Success
        } else {
            BehaviorResult::Fail
        }
    }
}

fn main() {
    let body = Body {
        left_arm: Arm {
            name: "left_arm".to_string(),
        },
        right_arm: Arm {
            name: "right_arm".to_string(),
        },
    };

    let mut ctx = Context::default();
    ctx.set("body", body);

    let mut root = SequenceNode::default();

    root.add_child(Box::new(PrintBodyNode), hash_map!());

    let mut print_arms = SequenceNode::default();
    print_arms.add_child(Box::new(PrintArmNode), hash_map!("arm" => "left_arm"));
    print_arms.add_child(Box::new(PrintArmNode), hash_map!("arm" => "right_arm"));

    root.add_child(Box::new(print_arms), hash_map!());

    root.tick(&mut ctx);
}
