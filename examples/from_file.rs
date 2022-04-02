use ::behavior_tree_lite::{
    load_yaml, BehaviorNode, BehaviorResult, Constructor, Context, Registry,
};
use std::fs;

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
            println!("No body!");
            BehaviorResult::Fail
        }
    }
}

struct PrintArmNodeConstructor;

impl Constructor for PrintArmNodeConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        Box::new(PrintArmNode)
    }
}

struct PrintBodyNodeConstructor;

impl Constructor for PrintBodyNodeConstructor {
    fn build(&self) -> Box<dyn BehaviorNode> {
        Box::new(PrintBodyNode)
    }
}

fn main() -> std::io::Result<()> {
    let mut registry = Registry::default();
    registry.register("PrintArmNode", Box::new(PrintArmNodeConstructor));
    registry.register("PrintBodyNode", Box::new(PrintBodyNodeConstructor));
    let file = String::from_utf8(fs::read("test.yaml")?).unwrap();
    if let Some(mut tree) = load_yaml(&file, &registry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
    {
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

        tree.tick(&mut ctx);
    }

    Ok(())
}
