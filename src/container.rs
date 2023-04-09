use std::{collections::HashMap, cell::Cell};

use crate::{
    error::{AddChildError, AddChildResult},
    BehaviorCallback, BehaviorNode, BehaviorResult, BlackboardValue, Context, NumChildren, Symbol, PortSpec, parser::PortMapOwned,
};

pub struct BehaviorNodeContainer {
    /// Name of the type of the node
    pub(crate) name: String,
    pub(crate) node: Box<dyn BehaviorNode>,
    pub(crate) blackboard_map: HashMap<Symbol, BlackboardValue>,
    pub(crate) child_nodes: Vec<BehaviorNodeContainer>,
    pub(crate) last_result: Option<BehaviorResult>,
    pub(crate) is_subtree: bool,
    pub(crate) subtree_expanded: Cell<bool>,
}

impl BehaviorNodeContainer {
    pub fn new(
        node: Box<dyn BehaviorNode>,
        blackboard_map: HashMap<Symbol, BlackboardValue>,
    ) -> Self {
        Self {
            name: "".to_owned(),
            node,
            blackboard_map,
            child_nodes: vec![],
            last_result: None,
            is_subtree: false,
            subtree_expanded: Cell::new(false),
        }
    }

    pub fn new_raw(node: Box<dyn BehaviorNode>) -> Self {
        Self {
            name: "".to_owned(),
            node,
            blackboard_map: HashMap::new(),
            child_nodes: vec![],
            last_result: None,
            is_subtree: false,
            subtree_expanded: Cell::new(false),
        }
    }

    pub fn new_node(node: impl BehaviorNode + 'static) -> Self {
        Self {
            name: "".to_owned(),
            node: Box::new(node),
            blackboard_map: HashMap::new(),
            child_nodes: vec![],
            last_result: None,
            is_subtree: false,
            subtree_expanded: Cell::new(false),
        }
    }

    pub(crate) fn new_raw_with_name(node: Box<dyn BehaviorNode>, name: String) -> Self {
        Self {
            name,
            node,
            blackboard_map: HashMap::new(),
            child_nodes: vec![],
            last_result: None,
            is_subtree: false,
            subtree_expanded: Cell::new(false),
        }
    }

    pub fn tick(&mut self, arg: BehaviorCallback, ctx: &mut Context) -> BehaviorResult {
        std::mem::swap(&mut self.child_nodes, &mut ctx.child_nodes.0);
        std::mem::swap(&mut self.blackboard_map, &mut ctx.blackboard_map);
        let res = self.node.tick(arg, ctx);
        std::mem::swap(&mut self.blackboard_map, &mut ctx.blackboard_map);
        std::mem::swap(&mut self.child_nodes, &mut ctx.child_nodes.0);
        res
    }

    pub fn add_child(&mut self, child: BehaviorNodeContainer) -> AddChildResult {
        if NumChildren::Finite(self.child_nodes.len()) < self.node.max_children() {
            self.child_nodes.push(child);
            Ok(())
        } else {
            Err(AddChildError::TooManyNodes)
        }
    }

    pub fn children(&self) -> &[BehaviorNodeContainer] {
        &self.child_nodes
    }

    pub fn last_result(&self) -> Option<BehaviorResult> {
        self.last_result
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn blackboard_map(&self) -> &HashMap<Symbol, BlackboardValue> {
        &self.blackboard_map
    }

    pub fn port_map<'a>(&'a self) -> impl Iterator<Item = PortMapOwned> {
        let items = self.node.provided_ports().into_iter().filter_map(|port| {
            if let Some(mapped) = self.blackboard_map.get(&port.key) {
                Some(PortMapOwned::new(port.ty,
                    port.key.to_string(), BlackboardValue::to_owned2(&mapped)
                ))
            } else {
                None
            }
        }).collect::<Vec<_>>();
        items.into_iter()
    }

    pub fn is_subtree(&self) -> bool {
        self.is_subtree
    }

    pub fn is_subtree_expanded(&self) -> bool {
        self.subtree_expanded.get()
    }

    pub fn expand_subtree(&self, b: bool) {
        self.subtree_expanded.set(b);
    }
}
