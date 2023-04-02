//! # behavior-tree-lite (Rust crate)
//!
//! An experimental Rust crate for minimal behavior tree implementation
//!
//!
//! ## Overview
//!
//! This is an implementation of behavior tree in Rust, inspired by [BehaviorTreeCPP](https://github.com/BehaviorTree/BehaviorTree.CPP.git).
//!
//! A behavior tree is an extension to finite state machines that makes describing transitional behavior easier.
//! See [BehaviorTreeCPP's documentation](https://www.behaviortree.dev/) for the thorough introduction to the idea.
//!
//! See the historical notes at the bottom of this README.md for more full history.
//!
//!
//! ## How it looks like
//!
//! First, you define the state with a data structure.
//!
//! ```rust
//! struct Arm {
//!     name: String,
//! }
//!
//! struct Body {
//!     left_arm: Arm,
//!     right_arm: Arm,
//! }
//!
//! let body = Body {
//!     left_arm: Arm {
//!         name: "leftArm".to_string(),
//!     },
//!     right_arm: Arm {
//!         name: "rightArm".to_string(),
//!     },
//! };
//! ```
//!
//! Then register the data to the context.
//!
//! ```rust
//! # use behavior_tree_lite::Context;
//! # let body = 1;
//! let mut ctx = Context::default();
//! ctx.set("body", body);
//! ```
//!
//! Then, you define a behavior tree.
//! Note that `add_child` method takes a mapping of blackboard variables as the second argument.
//!
//! ```rust
//! # use behavior_tree_lite::*;
//! # struct PrintBodyNode;
//! # impl BehaviorNode for PrintBodyNode { fn tick(&mut self, _: BehaviorCallback, _: &mut Context) -> BehaviorResult { BehaviorResult::Success }}
//! # struct PrintArmNode;
//! # impl BehaviorNode for PrintArmNode { fn tick(&mut self, _: BehaviorCallback, _: &mut Context) -> BehaviorResult { BehaviorResult::Success }}
//! let mut root = BehaviorNodeContainer::new_node(SequenceNode::default());
//! root.add_child(BehaviorNodeContainer::new_node(PrintBodyNode));
//!
//! let mut print_arms = BehaviorNodeContainer::new_node(SequenceNode::default());
//! print_arms.add_child(BehaviorNodeContainer::new(Box::new(PrintArmNode), hash_map!("arm" => "left_arm")));
//! print_arms.add_child(BehaviorNodeContainer::new(Box::new(PrintArmNode), hash_map!("arm" => "right_arm")));
//!
//! root.add_child(print_arms);
//! ```
//!
//! and call `tick()`.
//!
//! ```rust
//! # use behavior_tree_lite::*;
//! # let mut root = SequenceNode::default();
//! # let mut ctx = Context::default();
//! let result = root.tick(&mut |_| None, &mut ctx);
//! ```
//!
//! The first argument to the `tick` has weird value `&mut |_| None`.
//! It is a callback for the behavior nodes to communicate with the environment.
//! You could supply a closure to handle messages from the behavior nodes.
//! The closure, aliased as `BehaviorCallback`, takes a `&dyn std::any::Any` and returns a `Box<dyn std::any::Any>`,
//! which allows the user to pass or return any type, but in exchange, the user needs to
//! check the type with `downcast_ref` in order to use it like below.
//!
//! ```rust
//! # use behavior_tree_lite::*;
//! # let mut tree = SequenceNode::default();
//! tree.tick(
//!     &mut |v: &dyn std::any::Any| {
//!         println!("{}", *v.downcast_ref::<bool>().unwrap());
//!         None
//!     },
//!     &mut Context::default(),
//! );
//! ```
//!
//! This design was adopted because there is no other good ways to communicate between behavior nodes and the envrionment _whose lifetime is not 'static_.
//!
//! It is easy to communicate with global static variables, but users often want
//! to use behavior tree with limited lifetimes, like enemies' AI in a game.
//! Because you can't name a lifetime until you actually use the behavior tree,
//! you can't define a type that can send/receive data with arbitrary type having
//! lifetime shorter than 'static.
//! `std::any::Any` can't circumvent the limitation, because it is also bounded by 'static lifetime,
//! so as soon as you put your custom payload into it, you can't put any references other than `&'static`.
//!
//! With a closure, we don't have to name the lifetime and it will clearly outlive the duration of the closure body, so we can pass references around.
//!
//! Of course, you can also use blackboard variables, but they have the same limitation of lifetimes; you can't pass a reference through blackboard.
//! A callback is much more direct (and doesn't require indirection of port names)
//! way to communicate with the environment.
//!
//!
//! ## How to define your own node
//!
//! The core of the library is the `BehaviorNode` trait.
//! You can implement the trait to your own type to make a behavior node of your own.
//!
//! It is very similar to BehaviorTreeCPP.
//! For example a node to print the name of the arm can be defined like below.
//!
//! ```rust
//! # use behavior_tree_lite::*;
//! # #[derive(Debug)]
//! # struct Arm { name: String };
//! struct PrintArmNode;
//!
//! impl BehaviorNode for PrintArmNode {
//!     fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
//!         println!("Arm {:?}", ctx);
//!
//!         if let Some(arm) = ctx.get::<Arm>("arm") {
//!             println!("Got {}", arm.name);
//!         }
//!         BehaviorResult::Success
//!     }
//! }
//! ```
//!
//! In order to pass the variables, you need to set the variables to the blackboard.
//! This is done by `Context::set` method.
//!
//! ```rust
//! # use behavior_tree_lite::*;
//! # #[derive(Debug)]
//! # struct Body { left_arm: (), right_arm: () };
//! struct PrintBodyNode;
//!
//! impl BehaviorNode for PrintBodyNode {
//!     fn tick(&mut self, _arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
//!         if let Some(body) = ctx.get::<Body>("body") {
//!             let left_arm = body.left_arm.clone();
//!             let right_arm = body.right_arm.clone();
//!             println!("Got Body: {:?}", body);
//!             ctx.set("left_arm", left_arm);
//!             ctx.set("right_arm", right_arm);
//!             BehaviorResult::Success
//!         } else {
//!             BehaviorResult::Fail
//!         }
//!     }
//! }
//! ```
//!
//! ### Optimizing port access by caching symbols
//!
//! If you use ports a lot, you can try to minimize the cost of comparing and finding the port names as strings by using symbols.
//! Symbols are pointers that are guaranteed to compare equal if they point to the same string.
//! So you can simply compare the address to check the equality of them.
//!
//! You can use `Lazy<Symbol>` to use cache-on-first-use pattern on the symbol like below.
//! `Lazy` is re-exported type from `once_cell`.
//!
//! ```rust
//! # struct Body;
//! use ::behavior_tree_lite::{
//!     BehaviorNode, BehaviorResult, BehaviorCallback, Symbol, Lazy, Context
//! };
//!
//! struct PrintBodyNode;
//!
//! impl BehaviorNode for PrintBodyNode {
//!     fn tick(&mut self, _: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
//!         static BODY_SYM: Lazy<Symbol> = Lazy::new(|| "body".into());
//!         static LEFT_ARM_SYM: Lazy<Symbol> = Lazy::new(|| "left_arm".into());
//!         static RIGHT_ARM_SYM: Lazy<Symbol> = Lazy::new(|| "right_arm".into());
//!
//!         if let Some(body) = ctx.get::<Body>(*BODY_SYM) {
//!             // ...
//!             BehaviorResult::Success
//!         } else {
//!             BehaviorResult::Fail
//!         }
//!     }
//! }
//! ```
//!
//!
//! ### Provided ports
//!
//! You can declare what ports you would use in a node by defining `provided_ports` method.
//! This is optional, and only enforced if you specify `check_ports` argument in the `load` function explained later.
//! However, declaring provided_ports will help statically checking the code and source file consistency, so is generally encouraged.
//!
//! ```rust
//! use ::behavior_tree_lite::{
//!     BehaviorNode, BehaviorCallback, BehaviorResult, Context, Symbol, Lazy, PortSpec
//! };
//!
//! struct PrintBodyNode;
//!
//! impl BehaviorNode for PrintBodyNode {
//!     fn provided_ports(&self) -> Vec<PortSpec> {
//!         vec![
//!             PortSpec::new_in("body"),
//!             PortSpec::new_out("left_arm"),
//!             PortSpec::new_out("right_arm")
//!         ]
//!     }
//!
//!     fn tick(&mut self, _: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
//!         // ...
//!         BehaviorResult::Success
//!     }
//! }
//! ```
//!
//! See [example code](examples/main.rs) for the full code.
//!
//! ### Loading the tree structure from a yaml file
//!
//! Deprecated in favor of <a href="#The custom config file format">the custom config file format</a>.
//! It doesn't have much advantage over our custom format, except that it can be parsed by any yaml parser library (not limited to Rust).
//! However, parsing is only a small part of the whole process of loading dynamically configured behavior tree.
//! There are validation of port mapping and specifying input/output,
//! which is not any easier with yaml.
//! Our custom format has much more flexibility in load-time validation and
//! error handling.
//!
//! You can define the tree structure in a yaml file and configure it on runtime.
//! Yaml file is very good for writing human readable/writable configuration file.
//! It also looks similar to the actual tree structure.
//!
//! ```yaml
//! behavior_tree:
//!   type: Sequence
//!   children:
//!   - type: PrintBodyNode
//!   - type: Sequence
//!     children:
//!     - type: PrintArmNode
//!       ports:
//!         arm: left_arm
//!     - type: PrintArmNode
//!       ports:
//!         arm: right_arm
//! ```
//!
//! In order to load a tree from a yaml file, you need to register the node types
//! to the registry.
//!
//! ```rust
//! # use ::behavior_tree_lite::*;
//! # struct PrintBodyNode;
//! # impl BehaviorNode for PrintBodyNode { fn tick(&mut self, _: BehaviorCallback, _: &mut Context) -> BehaviorResult { BehaviorResult::Success }}
//! # struct PrintArmNode;
//! # impl BehaviorNode for PrintArmNode { fn tick(&mut self, _: BehaviorCallback, _: &mut Context) -> BehaviorResult { BehaviorResult::Success }}
//! let mut registry = Registry::default();
//! registry.register("PrintArmNode", boxify(|| PrintArmNode));
//! registry.register("PrintBodyNode", boxify(|| PrintBodyNode));
//! ```
//!
//! Some node types are registered by default, e.g. `SequenceNode` and `FallbackNode`.
//! ## The custom config file format
//!
//! We have specific file format for describing behavior tree structure of our own.
//! With this format, the same tree shown as YAML earlier can be written even more concisely like below.
//!
//! ```raw
//! tree main = Sequence {
//!   PrintBodyNode
//!   Sequence {
//!     PrintArmNode (arm <- left_arm)
//!     PrintArmNode (arm <- right_arm)
//!   }
//! }
//! ```
//!
//! It can be converted to an AST with `parse_file` function.
//! The AST (Abstract Syntax Tree) is a intermediate format of behavior tree,
//! from which you can instantiate actual behavior trees as many times as you want.
//! Note that the AST borrows lifetime of the argument string, so you cannot free the
//! source string before the AST.
//!
//! ```rust
//! # use ::behavior_tree_lite::*;
//! # let source_string = "";
//! # (|| -> Result<(&str, _), nom::error::Error<&str>> {
//! let (_, tree_source) = parse_file(source_string)?;
//! # Ok((source_string, tree_source))
//! # })();
//! ```
//!
//! and subsequently be instantiated to a tree.
//! The second argument `registry` is the same as yaml parser.
//! The third argument `check_ports` will switch port direction checking during loading.
//! If your `BehaviorNode::provided_ports` and the source file's direction arrow (`<-`, `->` or `<->`) disagree, it will become an error.
//!
//! ```rust
//! # use ::behavior_tree_lite::*;
//! # use ::nom::IResult;
//! # let source_string = "";
//! # let mut registry = Registry::default();
//! # let check_ports = true;
//! # (|| -> Result<(), error::LoadError> {
//! # let (_, tree_source) = parse_file(source_string).unwrap();
//! let tree = load(&tree_source, &registry, check_ports)?;
//! # Ok(())
//! # })();
//! ```
//!
//! ### Line comments
//!
//! You can put a line comment starting with a hash (`#`).
//!
//! ```raw
//! # This is a comment at the top level.
//!
//! tree main = Sequence { # This is a comment after opening brace.
//!            # This is a comment in a whole line.
//!     var a  # This is a comment after a variable declaration.
//!     Yes    # This is a comment after a node.
//! }          # This is a comment after a closing brace.
//! ```
//!
//!
//! ### Node definition
//!
//! A node can be specified like below.
//! It starts with a node name defined in Rust code.
//! After that, it can take an optional list of input/output ports in parentheses.
//!
//! ```raw
//! PrintString (input <- "Hello, world!")
//! ```
//!
//! The direction of the arrow in the ports specify input, output or inout port types.
//! The left hand side of the arrow is the port name defined in the node.
//! The right hand side is the blackboard variable name or a literal.
//!
//! ```raw
//! a <- b      input port
//! a -> b      output port
//! a <-> b     inout port
//! ```
//!
//! You can specify a literal string, surrounded by double quotes, to an input port,
//! but specifying a literal to output or inout node is an error.
//! The type of the literal is always a string, so if you want a number,
//! you may want to use `Context::get_parse()` method, which will automatically try
//! to parse from a string, if the type was not desired one.
//!
//! ```raw
//! a <- "Hi!"
//! a -> "Error!"
//! a <-> "Error too!"
//! ```
//!
//! It is an error to try to read from an output port or write to an input port,
//! but inout port can do both.
//!
//!
//! ### Child nodes
//!
//! A node can have a list of child nodes in braces.
//!
//! ```raw
//! Sequence {
//!     PrintString (input <- "First")
//!     PrintString (input <- "Second")
//! }
//! ```
//!
//! Or even both ports and children.
//!
//! ```raw
//! Repeat (n <- "100") {
//!     PrintString (input <- "Spam")
//! }
//! ```
//!
//! ### Subtrees
//!
//! It is very easy to define subtrees and organize your huge tree into modular structure.
//!
//! ```raw
//! tree main = Sequence {
//!     CanICallSubTree
//!     SubTree
//! }
//!
//! tree SubTree = Sequence {
//!     PrintString (input <- "Hi there!")
//! }
//! ```
//!
//! A subtree has its own namespace of blackboard variables.
//! It will keep the blackboard from being a huge table of global variables.
//!
//! If you need blackboard variables to communicate between the parent tree
//! and the subtree, you can put parentheses and a comma-separated list
//! after subtree name to specify the port definition of a subtree.
//!
//! A port "parameter" can be prefixed by either `in`, `out` or `inout`.
//! It will indicate the direction of data flow.
//!
//! The syntax is intentionally made similar to a function definition, because
//! it really is.
//!
//! ```raw
//! tree main = Sequence {
//!     SubTree (input <- "42", output -> subtreeResult)
//!     PrintString (input <- subtreeResult)
//! }
//!
//! tree SubTree(in input, out output) = Sequence {
//!     Calculate (input <- input, result -> output)
//! }
//! ```
//!
//!
//! ### Conditional syntax
//!
//! Like a programming language, the format supports conditional syntax.
//!
//! ```raw
//! tree main = Sequence {
//!     if (ConditionNode) {
//!         Yes
//!     }
//! }
//! ```
//!
//! If the `ConditionNode` returns `Success`, the inner braces are ticked
//! as a Sequence, otherwise it skips the nodes.
//!
//! It is not an `if` statement per se, because behavior tree does not have
//! the concept of a statement.
//! It is internally just a behavior node, having a special syntax for the
//! ease of editing and understanding.
//!
//! The code above desugars into this:
//!
//! ```raw
//! tree main = Sequence {
//!     if {
//!         ConditionNode
//!         Sequence {
//!             Yes
//!         }
//!     }
//! }
//! ```
//!
//! As you may expect, you can put `else` clause.
//!
//! ```raw
//! tree main = Sequence {
//!     if (ConditionNode) {
//!         Yes
//!     } else {
//!         No
//!     }
//! }
//! ```
//!
//! `if` is a built-in node type, which can take 2 or 3 child nodes.
//! The first child is the condition, the second is the `then` clause, and
//! the optional third child is the `else` clause.
//! `then` and `else` clause are implicitly wrapped in a `Sequence`.
//!
//! The syntax also supports negation operator (`!`) in front of the condition node.
//! This code below is
//!
//! ```raw
//! tree main = Sequence {
//!     if (!ConditionNode) {
//!         Yes
//!     }
//! }
//! ```
//!
//! equivalent to this one:
//!
//! ```raw
//! tree main = Sequence {
//!     if (ConditionNode) {
//!     } else {
//!         Yes
//!     }
//! }
//! ```
//!
//! You can put logical operators (`&&` and `||`) like conditional expressions in programming languages.
//! `&&` is just a shorthand for a Sequence node and `||` is a Fallback node.
//!
//! ```raw
//! tree main = Sequence {
//!     if (!a || b && c) {}
//! }
//! ```
//!
//! In fact, a child node is implicitly a logical expression, so you can write like this:
//!
//! ```raw
//! tree main = Sequence {
//!     !a || b && c
//! }
//! ```
//!
//! Parentheses can be used to group operators.
//!
//! ```raw
//! tree main = Sequence {
//!     (!a || b) && c
//! }
//! ```
//!
//! `if` node without else clause is semantically the same as a Sequence node like below,
//! but Sequence or Fallback nodes cannot represent `else` clause easily.
//!
//! ```raw
//! tree main = Sequence {
//!     Sequence {
//!         ConditionNode
//!         Sequence {
//!             Yes
//!         }
//!     }
//! }
//! ```
//!
//!
//! ### Blackboard variable declarations
//!
//! You can optionally declare and initialize a blackboard variable.
//! It can be used as a node name, and its value is evaluated as a boolean.
//! So, you can put the variable into a `if` condition.
//!
//! ```raw
//! tree main = Sequence {
//!     var flag = true
//!     if (flag) {
//!         Yes
//!     }
//! }
//! ```
//!
//! Currently, only `true` or `false` is allowed as the initializer (the right hand side of `=`).
//!
//! The variable declaration with initialization will desugar into a `SetBool` node.
//! A reference to a variable name will desugar into a `IsTrue` node.
//!
//! ```raw
//! tree main = Sequence {
//!     SetBool (value <- "true", output -> flag)
//!     if (IsTrue (input <- flag)) {
//!         Yes
//!     }
//! }
//! ```
//!
//! However, it is necessary to declare the variable name in order to use it as a
//! variable reference.
//! For example, the code below will be a `load` error for `MissingNode`, even though
//! the variable is set with `SetBool`.
//!
//! ```raw
//! tree main = Sequence {
//!     SetBool (value <- "true", output -> flag)
//!     if (flag) {
//!         Yes
//!     }
//! }
//! ```
//!
//! This design is a step towards statically checked source code.
//!
//!
//!
//! ### Syntax specification
//!
//! Here is a pseudo-EBNF notation of the syntax.
//!
//! Note that this specification is by no means accurate EBNF.
//! The syntax is defined by recursive descent parser with parser combinator,
//! which removes ambiguity, but this EBNF may have ambiguity.
//!
//! ```raw
//! tree = "tree" tree-name [ "(" tree-port-list ")" ] "=" node
//!
//! tree-port-list = port-def | tree-port-list "," port-def
//!
//! port-def = ( "in" | "out" | "inout" ) tree-port-name
//!
//! tree-port-name = identifier
//!
//! node = if-syntax | conditional | var-def-syntax | var-assign
//!
//! if-syntax = "if" "(" conditional ")"
//!
//! conditional-factor = "!" conditional-factor | node-syntax
//!
//! conditional-and =  conditional-factor | conditional "&&" conditional-factor
//!
//! conditional =  conditional-and | conditional "||" conditional-and
//!
//! node-syntax = node-name [ "(" port-list ")" ] [ "{" node* "}" ]
//!
//! port-list = port [ "," port-list ]
//!
//! port = node-port-name ("<-" | "->" | "<->") blackboard-port-name
//!
//! node-port-name = identifier
//!
//! blackboard-port-name = identifier
//!
//! var-def-syntax = "var" identifier "=" initializer
//!
//! var-assign = identifier "=" initializer
//!
//! initializer = "true" | "false"
//! ```
//!
//!
//! ## TODO
//!
//! * [x] Easier way to define constructors (macros?)
//! * [ ] Full set of control nodes
//!   * [x] Reactive nodes
//!   * [ ] Star nodes
//!   * [x] Decorator nodes
//! * [x] Performance friendly blackboard keys
//! * [x] DSL for defining behavior tree structure
//!   * [x] Programming language-like flow control syntax
//! * [ ] Static type checking for behavior tree definition file
//!
//! # Historical notes
//!
//! This is a sister project of [tiny-behavior-tree](https://github.com/msakuta/rusty_tiny_behavior_tree) which in turn inspired by [BehaviorTreeCPP](https://github.com/BehaviorTree/BehaviorTree.CPP.git).
//!
//! While tiny-behavior-tree aims for more innovative design and experimental features, this crate aims for more traditional behavior tree implementation.
//! The goal is to make a crate lightweight enough to use in WebAssembly.
//!
//! ## The difference from tiny-behavior-tree
//!
//! The main premise of tiny-behavior-tree is that it passes data with function arguments.
//! This is very good for making fast and small binary, but it suffers from mixed node types in a tree.
//!
//! It requires ugly boilerplate code or macros to convert types between different node argument types, and there is the concept of "PeelNode" which would be unnecessary in traditional behavior tree design.
//!
//! On top of that, uniform types make it much easier to implement configuration file parser that can change the behavior tree at runtime.
//!
//!
//! ## Performance consideration
//!
//! One of the issues with behavior tree in general regarding performance is that the nodes communicate with blackboard variables, which is essentially a key-value store.
//! It is not particularly bad, but if you read/write a lot of variables in the blackboard (which easily happens with a large behavior tree), you would pay the cost of constructing a string and looking up HashMap every time.
//!
//! One of the tiny-behavior-tree's goals is to address this issue by passing variables with function call arguments.
//! Why would you pay the cost of looking up HashMap if you already know the address of the variable?
//!
//! Also, the blackboard is not very scalable, since it is essentially a huge table of global variables.
//! Although there is sub-blackboards in subtrees, it is difficult to keep track of similar to scripting language's stack frame without proper debugging tools.
//!
//! I might experiment with non-string keys to make it more efficient, but the nature of the variables need to be handled dynamically in uniformly typeds nodes.

mod container;
mod context;
pub mod error;
mod nodes;
pub mod parser;
mod port;
mod registry;
mod symbol;

use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::container::BehaviorNodeContainer;
pub use crate::context::Context;
pub use crate::nodes::{tick_child_node, FallbackNode, SequenceNode};
pub use crate::symbol::Symbol;
pub use crate::{
    parser::{load, load_yaml, node_def, parse_file, parse_nodes, NodeDef},
    port::{PortSpec, PortType},
    registry::{boxify, Constructor, Registry},
};
pub use ::once_cell::sync::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
    /// The node should keep running in the next tick
    Running,
}

#[derive(Debug)]
pub enum BlackboardValue {
    Ref(Symbol, PortType),
    Literal(String),
}

impl From<&str> for BlackboardValue {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_owned())
    }
}

/// Blackboard is a mapping of a variable names and their values.
/// The value is wrapped in an `Any` trait object, so it can be any type.
///
/// # Implementation note
///
/// You may wonder why the value is wrapped in an `Rc`, not a `Box`.
/// It seems unnecessary to have reference count for owned values inside a
/// blackboard.
/// The reason is that we need to pass the copy of the variables to the subtree by
/// copying values, but `Clone` trait is not object safe.
///
/// Another way to work around this is to define a new trait, like `AnyClone`,
/// that can clone and be object safe at the same time.
/// The signature for the clone method would be something like:
///
/// ```
/// # use std::any::Any;
/// trait AnyClone: Any {
///     fn any_clone(&self) -> Box<dyn AnyClone>;
/// }
/// ```
///
/// The difference from `Clone` trait is that it returns a boxed copy of the object
/// to avoid the requirement that returned value of a function needs to be `Sized`.
/// However, requiring every type that can exist in the blackboard to implement this
/// trait is a bit too much to ask to the users.
/// By wrapping in an `Rc`, it looks like cloneable, but it doesn't require the value
/// type to implement anything but `Any`.
///
/// Interestingly, this is the exactly the same issue when you try to implement
/// a function call in your own programming language, i.e. pass-by-value v.s. pass-by-reference.
/// In essence, a subtree in behavior tree is a function in a programming language.
/// The third sect in the society is copy-on-write reference, which is what `Rc` does.
pub type Blackboard = HashMap<Symbol, Rc<dyn Any>>;
pub type BBMap = HashMap<Symbol, BlackboardValue>;
pub type BehaviorCallback<'a> = &'a mut dyn FnMut(&dyn Any) -> Option<Box<dyn Any>>;

#[derive(PartialEq, Eq)]
pub enum NumChildren {
    Finite(usize),
    Infinite,
}

impl PartialOrd for NumChildren {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match (self, other) {
            (NumChildren::Finite(_), NumChildren::Infinite) => std::cmp::Ordering::Less,
            (NumChildren::Infinite, NumChildren::Finite(_)) => std::cmp::Ordering::Greater,
            (NumChildren::Finite(lhs), NumChildren::Finite(rhs)) => lhs.cmp(rhs),
            (NumChildren::Infinite, NumChildren::Infinite) => return None,
        })
    }
}

pub trait BehaviorNode {
    fn provided_ports(&self) -> Vec<PortSpec> {
        vec![]
    }

    fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult;

    fn num_children(&self) -> NumChildren {
        NumChildren::Finite(0)
    }
}

#[macro_export]
macro_rules! hash_map {
    () => {
        std::collections::HashMap::default()
    };
    ($name: literal => $val: expr) => {{
        let mut ret = std::collections::HashMap::default();
        ret.insert($name.into(), $val.into());
        ret
    }};
}
