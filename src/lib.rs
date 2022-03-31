use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BehaviorResult {
    Success,
    Fail,
}

#[derive(Default, Debug)]
pub struct Context {
    blackboard: HashMap<String, Box<dyn std::any::Any>>,
    blackboard_map: HashMap<String, String>,
}

impl Context {
    pub fn get<'a, T: 'static>(&'a self, key: &str) -> Option<&'a T> {
        let mapped = self
            .blackboard_map
            .get(key)
            .map(|key| key as &str)
            .unwrap_or(key);
        // println!("mapped: {:?}", mapped);
        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<S: ToString, T: 'static>(&mut self, key: S, val: T) {
        let key = key.to_string();
        let mapped = self.blackboard_map.get(&key).cloned().unwrap_or(key);
        self.blackboard.insert(mapped, Box::new(val));
    }
}

pub trait BehaviorNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult;
}

pub struct BehaviorNodeContainer {
    node: Box<dyn BehaviorNode>,
    blackboard_map: HashMap<String, String>,
}

#[derive(Default)]
pub struct SequenceNode {
    children: Vec<BehaviorNodeContainer>,
}

impl SequenceNode {
    pub fn add_child<T: BehaviorNode + 'static>(
        &mut self,
        val: T,
        blackboard_map: HashMap<String, String>,
    ) {
        self.children.push(BehaviorNodeContainer {
            node: Box::new(val),
            blackboard_map,
        });
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Fail {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }
}

#[derive(Default)]
pub struct FallbackNode {
    children: Vec<BehaviorNodeContainer>,
}

impl FallbackNode {
    pub fn add_child<T: BehaviorNode + 'static>(
        &mut self,
        val: T,
        blackboard_map: HashMap<String, String>,
    ) {
        self.children.push(BehaviorNodeContainer {
            node: Box::new(val),
            blackboard_map,
        });
    }
}

impl BehaviorNode for FallbackNode {
    fn tick(&mut self, ctx: &mut Context) -> BehaviorResult {
        for node in &mut self.children {
            std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
            if node.node.tick(ctx) == BehaviorResult::Fail {
                std::mem::swap(&mut ctx.blackboard_map, &mut node.blackboard_map);
                return BehaviorResult::Fail;
            }
        }
        BehaviorResult::Success
    }
}

#[macro_export]
macro_rules! hash_map {
    () => {
        std::collections::HashMap::default()
    };
    ($name: literal => $val: expr) => {{
        let mut ret = std::collections::HashMap::default();
        ret.insert($name.to_string(), $val.to_string());
        ret
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
