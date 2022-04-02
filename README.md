# behavior-tree-lite (Rust crate)

An experimental Rust crate for minimal behavior tree implementation


## Overview

This is a sister project of [tiny-behavior-tree](https://github.com/msakuta/rusty_tiny_behavior_tree) which in turn inspired by [BehaviorTreeCPP](https://github.com/BehaviorTree/BehaviorTree.CPP.git).

While tiny-behavior-tree aims for more innovative design and experimental features, this crate aims for more traditional behavior tree implementation.
However, we are not going to implement advanced features such as asynchronous nodes or coroutines.
My goal is to make a crate lightweight enough to use in WebAssembly.

## The difference from tiny-behavior-tree

The main premise of tiny-behavior-tree is that it passes data with function arguments.
This is very good for making fast and small binary, but it suffers from mixed node types in a tree.

It requires ugly boilerplate code or macros to convert types between different node argument types, and there is the concept of "PeelNode" which would be unnecessary in traditional behavior tree design.

On top of that, uniform types make it much easier to implement configuration file parser that can change the behavior tree at runtime.


## Performance consideration

One of the issues with behavior tree in general regarding performance is that the nodes communicate with blackboard variables, which is essentially a key-value store.
It is not particularly bad, but if you read/write a lot of variables in the blackboard (which easily happens with a large behavior tree), you would pay the cost of constructing a string and looking up HashMap every time.

One of the tiny-behavior-tree's goals is to address this issue by passing variables with function call arguments.
Why would you pay the cost of looking up HashMap if you already know the address of the variable?

Also, the blackboard is not very scalable, since it is essentially a huge table of global variables.
Although there is sub-blackboards in subtrees, it is difficult to keep track of similar to scripting language's stack frame without proper debugging tools.

I might experiment with non-string keys to make it more efficient, but the nature of the variables need to be handled dynamically in uniformly typeds nodes.


## How it looks like

The usage is very similar to TinyBehaviorTree.

First, you define the state with a data structure.

```rust
struct Arm {
    name: String,
}

struct Body {
    left_arm: Arm,
    right_arm: Arm,
}

let body = Body {
    left_arm: Arm {
        name: "leftArm".to_string(),
    },
    right_arm: Arm {
        name: "rightArm".to_string(),
    },
};
```

Then register the data to the context.

```rust
let mut ctx = Context::default();
ctx.set("body", body);
```

Then, you define a behavior tree.
Note that `add_child` method takes a mapping of blackboard variables as the second argument.

```rust
let mut root = SequenceNode::default();
root.add_child(Box::new(PrintBodyNode), hash_map!());

let mut print_arms = SequenceNode::default();
print_arms.add_child(Box::new(PrintArmNode), hash_map!("arm" => "left_arm"));
print_arms.add_child(Box::new(PrintArmNode), hash_map!("arm" => "right_arm"));

root.add_child(Box::new(print_arms), hash_map!());
```

and call `tick()`.

```rust
let result = root.tick(&mut ctx);
```

## How to define your own node

The core of the library is the `BehaviorNode` trait.
You can implement the trait to your own type to make a behavior node of your own.

It is very similar to BehaviorTreeCPP.
For example a node to print the name of the arm can be defined like below.

```rust
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
```

In order to pass the variables, you need to set the variables to the blackboard.
This is done by `Context::set` method.

```rust
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
```

See [example code](examples/main.rs) for the full code.

### Loading the tree structure from a yaml file

You can define the tree structure in a yaml file and configure it on runtime.
Yaml file is very good for writing human readable/writable configuration file.
It also looks similar to the actual tree structure.

```yaml
behavior_tree:
  type: Sequence
  children:
  - type: PrintBodyNode
  - type: Sequence
    children:
    - type: PrintArmNode
      ports:
        arm: left_arm
    - type: PrintArmNode
      ports:
        arm: right_arm
```

In order to load a tree from a yaml file, you need to register the node types
to the registry.

```rust
let mut registry = Registry::default();
registry.register("PrintArmNode", Box::new(PrintArmNodeConstructor));
registry.register("PrintBodyNode", Box::new(PrintBodyNodeConstructor));
```

Some node types are registered by default, e.g. `SequenceNode` and `FallbackNode`.


## TODO

* [ ] Easier way to define constructors (macros?)
* [ ] Full set of control nodes
* [ ] Performance friendly blackboard keys
