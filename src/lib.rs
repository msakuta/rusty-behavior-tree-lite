use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
}

#[derive(Default)]
pub struct Context {
    blackboard: HashMap<String, Box<dyn std::any::Any>>,
}

impl Context {
    pub fn get<'a, T: 'static>(&'a self, key: &str) -> Option<&'a T> {
        self.blackboard.get(key).and_then(|val| val.downcast_ref())
    }

    pub fn set<S: ToString, T: 'static>(&mut self, key: S, val: T) {
        self.blackboard.insert(key.to_string(), Box::new(val));
    }
}

pub trait BehaviorNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult;
}

#[derive(Default)]
pub struct SequenceNode {
    children: Vec<Box<dyn BehaviorNode>>,
}

impl SequenceNode {
    pub fn add_child<T: BehaviorNode + 'static>(&mut self, val: T) {
        self.children.push(Box::new(val));
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            if node.tick(ctx) == BehaviorResult::Fail {
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
