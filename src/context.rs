use crate::{BBMap, Blackboard, BlackboardValue, Symbol};
use std::{any::Any, str::FromStr};

#[derive(Default, Debug)]
pub struct Context<'e, T: 'e = ()> {
    blackboard: Blackboard,
    pub(crate) blackboard_map: BBMap,
    pub env: Option<&'e mut T>,
}

impl<'e, T> Context<'e, T> {
    pub fn new(blackboard: Blackboard) -> Self {
        Self {
            blackboard,
            blackboard_map: BBMap::new(),
            env: None,
        }
    }

    pub fn take_blackboard(self) -> Blackboard {
        self.blackboard
    }
}

impl<'e, E> Context<'e, E> {
    pub fn get<'a, T: 'static>(&'a self, key: impl Into<Symbol>) -> Option<&'a T>
    where
        'e: 'a,
    {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => &key,
            Some(BlackboardValue::Ref(mapped)) => mapped,
            Some(BlackboardValue::Literal(mapped)) => {
                return (mapped as &dyn Any).downcast_ref();
            }
        };

        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    pub fn set<T: 'static>(&mut self, key: impl Into<Symbol>, val: T) {
        let key = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => key,
            Some(BlackboardValue::Ref(mapped)) => *mapped,
            Some(BlackboardValue::Literal(_)) => panic!("Cannot write to a literal!"),
        };
        self.blackboard.insert(mapped, Box::new(val));
    }

    // pub fn get_env(&mut self) -> Option<&mut E> {
    //     self.env
    // }
}
