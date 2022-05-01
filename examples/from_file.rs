use ::behavior_tree_lite::{
    load_yaml, BehaviorNode, BehaviorResult, Constructor, Context, Registry,
};
use std::fs;
use symbol::Symbol;

#[derive(Clone, Debug)]
struct Arm {
    name: String,
}

#[derive(Debug)]
struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

struct PrintArmNode {
    arm_sym: Symbol,
}

impl PrintArmNode {
    fn new() -> Self {
        Self {
            arm_sym: "arm".into(),
        }
    }
}

impl BehaviorNode for PrintArmNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        println!("Arm {:?}", ctx);

        if let Some(arm) = ctx.get::<Arm>(self.arm_sym) {
            println!("Got {}", arm.name);
        }
        BehaviorResult::Success
    }
}

struct PrintBodyNode {
    body_sym: Symbol,
    left_arm_sym: Symbol,
    right_arm_sym: Symbol,
}

impl PrintBodyNode {
    fn new() -> Self {
        Self {
            body_sym: "body".into(),
            left_arm_sym: "left_arm".into(),
            right_arm_sym: "right_arm".into(),
        }
    }
}

impl BehaviorNode for PrintBodyNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        if let Some(body) = ctx.get::<Body>(self.body_sym) {
            let left_arm = body.left_arm.clone();
            let right_arm = body.right_arm.clone();
            println!("Got Body: {:?}", body);
            ctx.set(self.left_arm_sym, left_arm);
            ctx.set(self.right_arm_sym, right_arm);
            BehaviorResult::Success
        } else {
            println!("No body!");
            BehaviorResult::Fail
        }
    }
}

struct PrintArmNodeConstructor;

impl Constructor for PrintArmNodeConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        Box::new(PrintArmNode::new())
    }
}

struct PrintBodyNodeConstructor;

impl Constructor for PrintBodyNodeConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        let start = std::time::Instant::now();

        let ret = PrintBodyNode::new();

        eprintln!(
            "construct time: {}",
            start.elapsed().as_nanos() as f64 * 1e-9
        );

        Box::new(ret)
    }
}

fn main() -> anyhow::Result<()> {
    let mut registry = Registry::default();
    registry.register("PrintArmNode", Box::new(PrintArmNodeConstructor));
    registry.register("PrintBodyNode", Box::new(PrintBodyNodeConstructor));
    let file = String::from_utf8(fs::read("test.yaml")?).unwrap();
    let mut trees = load_yaml(&file, &registry)?;

    if let Some(main) = trees.get_mut("main") {
        let body = Body {
            left_arm: Arm {
                name: "left_arm".to_string(),
            },
            right_arm: Arm {
                name: "right_arm".to_string(),
            },
        };

        let mut ctx = Context::default();
        ctx.set("body".into(), body);

        let result = main.tick(&mut ctx);

        eprintln!("result: {:?}", result);
    }

    Ok(())
}
