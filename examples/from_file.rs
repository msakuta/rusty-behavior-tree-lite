use ::behavior_tree_lite::{
    load, parse_file, BehaviorCallback, BehaviorNode, BehaviorResult, Context, Registry,
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
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        println!("Arm {:?}", ctx);

        if let Some(arm) = ctx.get::<Arm>(self.arm_sym) {
            println!("Got {}", arm.name);
        }
        BehaviorResult::Success
    }
}

struct PrintStringNode {
    input: Symbol,
}

impl PrintStringNode {
    fn new() -> Self {
        Self {
            input: "input".into(),
        }
    }
}

impl BehaviorNode for PrintStringNode {
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        if let Some(s) = ctx.get::<String>(self.input) {
            println!("PrintStringNode: {}", s);
        } else {
            println!("PrintStringNode: didn't get string");
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
    fn tick(&mut self, _: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
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

fn print_arm_node() -> Box<dyn BehaviorNode> {
    Box::new(PrintArmNode::new())
}

fn print_body_node() -> Box<dyn BehaviorNode> {
    let start = std::time::Instant::now();

    let ret = PrintBodyNode::new();

    eprintln!(
        "construct time: {}",
        start.elapsed().as_nanos() as f64 * 1e-9
    );

    Box::new(ret)
}

fn main() -> anyhow::Result<()> {
    let mut registry = Registry::default();
    registry.register("PrintArmNode", Box::new(print_arm_node));
    registry.register("PrintBodyNode", Box::new(print_body_node));
    registry.register(
        "PrintStringNode",
        Box::new(|| Box::new(PrintStringNode::new())),
    );

    let file = String::from_utf8(fs::read("test.txt")?).unwrap();

    let (_, tree_source) =
        parse_file(&file).map_err(|e| anyhow::format_err!("parse error: {e:?}"))?;
    println!("tree_source: {tree_source:#?}");

    // if let Some(main) = trees.get_mut("main") {
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

    let mut root =
        load(&tree_source, &registry).map_err(|e| anyhow::format_err!("parse error: {e}"))?;
    let mut null = |_: &dyn std::any::Any| -> Option<Box<dyn std::any::Any>> { None };
    println!("root: {:?}", root.tick(&mut null, &mut ctx));

    //     let result = main.tick(&mut ctx);

    //     eprintln!("result: {:?}", result);
    // }

    Ok(())
}
