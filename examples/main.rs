use ::behavior_tree_lite::{
    hash_map, BehaviorCallback, BehaviorNode, BehaviorResult, BlackboardValue, Context, Lazy,
    SequenceNode, Symbol,
};

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
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        static ARM_SYM: Lazy<Symbol> = Lazy::new(|| "arm".into());
        if let Some(arm) = ctx.get::<Arm>(*ARM_SYM) {
            println!("PrintArmNode: {}", arm.name);
        }
        BehaviorResult::Success
    }
}

struct PrintBodyNode;

impl BehaviorNode for PrintBodyNode {
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        static BODY_SYM: Lazy<Symbol> = Lazy::new(|| "body".into());
        static LEFT_ARM_SYM: Lazy<Symbol> = Lazy::new(|| "left_arm".into());
        static RIGHT_ARM_SYM: Lazy<Symbol> = Lazy::new(|| "right_arm".into());
        if let Some(body) = ctx.get::<Body>(*BODY_SYM) {
            let left_arm = body.left_arm.clone();
            let right_arm = body.right_arm.clone();
            println!("PrintBodyNode: {:?}", body);
            ctx.set(*LEFT_ARM_SYM, left_arm);
            ctx.set(*RIGHT_ARM_SYM, right_arm);
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

    root.add_child(Box::new(PrintBodyNode), hash_map!())
        .unwrap();

    let mut print_arms = SequenceNode::default();
    print_arms
        .add_child(
            Box::new(PrintArmNode),
            hash_map!("arm" => BlackboardValue::Ref("left_arm".into())),
        )
        .unwrap();
    print_arms
        .add_child(
            Box::new(PrintArmNode),
            hash_map!("arm" => BlackboardValue::Ref("right_arm".into())),
        )
        .unwrap();

    root.add_child(Box::new(print_arms), hash_map!()).unwrap();

    root.tick(&mut |_| None, &mut ctx);

    println!("Total symbols: {}", Symbol::count());
}
