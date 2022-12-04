use super::*;
use crate::{boxify, error::LoadError, BehaviorResult, Context};

struct PrintNode;

impl BehaviorNode for PrintNode {
    fn tick(
        &mut self,
        arg: crate::BehaviorCallback,
        _ctx: &mut crate::Context,
    ) -> crate::BehaviorResult {
        arg(&42);
        BehaviorResult::Success
    }
}

#[test]
fn test_subtree() {
    let tree = r#"
tree main = Sequence {
    sub
}

tree sub = Fallback {
    PrintNode
}
    "#;

    let (_, tree_source) = crate::parse_file(tree).unwrap();
    let mut registry = Registry::default();
    registry.register("PrintNode", boxify(|| PrintNode));
    let mut tree = load(&tree_source, &registry, true).unwrap();

    let mut values = vec![];
    let result = tree.tick(
        &mut |val| {
            val.downcast_ref::<i32>().map(|val| values.push(*val));
            None
        },
        &mut Context::default(),
    );
    assert_eq!(result, BehaviorResult::Success);
    assert_eq!(values, vec![42]);
}

struct SendToArg;

impl BehaviorNode for SendToArg {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in("input")]
    }

    fn tick(&mut self, arg: crate::BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let input = ctx.get_parse::<i32>("input").unwrap();
        arg(&input);
        BehaviorResult::Success
    }
}

#[test]
fn test_subtree_map() {
    let tree = r#"
tree main = Sequence {
sub(input <- "96")
}

tree sub(in input, out output) = Fallback {
SendToArg (input <- input)
}
"#;
    let (_, tree_source) = crate::parse_file(tree).unwrap();
    let mut registry = Registry::default();
    registry.register("SendToArg", boxify(|| SendToArg));
    let mut tree = load(&tree_source, &registry, true).unwrap();

    let mut values = vec![];
    let result = tree.tick(
        &mut |val| {
            val.downcast_ref::<i32>().map(|val| values.push(*val));
            None
        },
        &mut Context::default(),
    );
    assert_eq!(result, BehaviorResult::Success);
    assert_eq!(values, vec![96]);
}

struct DoubleNode;

impl BehaviorNode for DoubleNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::new_in("input"), PortSpec::new_out("output")]
    }

    fn tick(&mut self, _arg: crate::BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        let input = ctx.get_parse::<i32>("input").unwrap();
        ctx.set("output", input * 2);
        BehaviorResult::Success
    }
}

#[test]
fn test_subtree_output() {
    let tree = r#"
tree main = Sequence {
sub(input <- "42", output -> doubled)
SendToArg (input <- doubled)
}

tree sub(in input, out output) = Fallback {
Double (input <- input, output -> output)
}
"#;
    let (_, tree_source) = crate::parse_file(tree).unwrap();
    let mut registry = Registry::default();
    registry.register("SendToArg", boxify(|| SendToArg));
    registry.register("Double", boxify(|| DoubleNode));
    let mut tree = load(&tree_source, &registry, true).unwrap();

    let mut values = vec![];
    let result = tree.tick(
        &mut |val| {
            val.downcast_ref::<i32>().map(|val| values.push(*val));
            None
        },
        &mut Context::default(),
    );
    assert_eq!(result, BehaviorResult::Success);
    assert_eq!(values, vec![84]);
}

#[test]
fn recurse() {
    let (_, st) = crate::parse_file(
        "
tree main = Sequence {
Sub
}

tree Sub = Sequence {
Sub
}
    ",
    )
    .unwrap();

    assert!(matches!(
        load(&st, &Registry::default(), false),
        Err(LoadError::InfiniteRecursion { .. })
    ));
}
