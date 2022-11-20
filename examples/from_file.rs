use ::behavior_tree_lite::{
    boxify, load, parse_file, BehaviorCallback, BehaviorNode, BehaviorResult, Context, Lazy,
    Registry, Symbol,
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
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        static ARM_SYM: Lazy<Symbol> = Lazy::new(|| "arm".into());
        if let Some(arm) = ctx.get::<Arm>(*ARM_SYM) {
            println!("PrintArmNode: {}", arm.name);
        }
        BehaviorResult::Success
    }
}

struct PrintStringNode;

impl BehaviorNode for PrintStringNode {
    fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        static INPUT: Lazy<Symbol> = Lazy::new(|| "input".into());
        if let Some(s) = ctx.get::<String>(*INPUT) {
            println!("PrintStringNode: {}", s);
        } else {
            println!("PrintStringNode: didn't get string");
        }
        BehaviorResult::Success
    }
}

struct PrintBodyNode;

impl BehaviorNode for PrintBodyNode {
    fn tick(&mut self, _: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
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
            println!("No body!");
            BehaviorResult::Fail
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut registry = Registry::default();
    registry.register("PrintArmNode", boxify(|| PrintArmNode));
    registry.register("PrintBodyNode", boxify(|| PrintBodyNode));
    registry.register("PrintStringNode", boxify(|| PrintStringNode));

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
    ctx.set("body", body);

    let mut root = load(&tree_source, &registry, false)
        .map_err(|e| anyhow::format_err!("parse error: {e}"))?;
    let mut null = |_: &dyn std::any::Any| -> Option<Box<dyn std::any::Any>> { None };
    println!("root: {:?}", root.tick(&mut null, &mut ctx));

    println!("Total symbols: {}", Symbol::count());

    Ok(())
}
