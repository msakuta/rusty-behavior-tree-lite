use crate::{BBMap, Blackboard, BlackboardValue, PortType, Symbol};
use std::{any::Any, rc::Rc, str::FromStr};

#[derive(Default, Debug)]
pub struct Context<'e, T: 'e = ()> {
    pub(crate) blackboard: Blackboard,
    pub(crate) blackboard_map: BBMap,
    pub env: Option<&'e mut T>,
    strict: bool,
}

impl<'e, T> Context<'e, T> {
    pub fn new(blackboard: Blackboard) -> Self {
        Self {
            blackboard,
            blackboard_map: BBMap::new(),
            env: None,
            strict: true,
        }
    }

    pub fn take_blackboard(self) -> Blackboard {
        self.blackboard
    }

    pub fn strict(&self) -> bool {
        self.strict
    }

    pub fn set_strict(&mut self, b: bool) {
        self.strict = b;
    }
}

impl<'e, E> Context<'e, E> {
    /// Get a blackboard variable with downcasting to the type argument.
    /// Returns `None` if it fails to downcast.
    pub fn get<'a, T: 'static>(&'a self, key: impl Into<Symbol>) -> Option<&'a T>
    where
        'e: 'a,
    {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => &key,
            Some(BlackboardValue::Ref(mapped, ty)) => {
                if matches!(*ty, PortType::Input | PortType::InOut) {
                    mapped
                } else {
                    if self.strict {
                        panic!("Port {:?} is not specified as input or inout port", key);
                    }
                    return None;
                }
            }
            Some(BlackboardValue::Literal(mapped)) => {
                return (mapped as &dyn Any).downcast_ref();
            }
        };

        self.blackboard.get(mapped).and_then(|val| {
            // println!("val: {:?}", val);
            val.downcast_ref()
        })
    }

    /// Get a blackboard variable without downcasting.
    pub fn get_any(&self, key: impl Into<Symbol>) -> Option<Rc<dyn Any>> {
        let key: Symbol = key.into();
        let mapped = self.blackboard_map.get(&key);
        let mapped = match mapped {
            None => &key,
            Some(BlackboardValue::Ref(mapped, ty)) => {
                if matches!(*ty, PortType::Input | PortType::InOut) {
                    mapped
                } else {
                    if self.strict {
                        panic!("Port {:?} is not specified as input or inout port", key);
                    }
                    return None;
                }
            }
            Some(BlackboardValue::Literal(mapped)) => {
                return Some(Rc::new(mapped.clone()));
            }
        };

        self.blackboard.get(mapped).cloned()
    }

    /// Convenience method to get raw primitive types such as f64 or parse from string
    pub fn get_parse<F>(&self, key: impl Into<Symbol> + Copy) -> Option<F>
    where
        F: FromStr + Copy + 'static,
    {
        self.get::<F>(key).copied().or_else(|| {
            self.get::<String>(key)
                .and_then(|val| val.parse::<F>().ok())
        })
    }

    pub fn set<T: 'static>(&mut self, key: impl Into<Symbol>, val: T) {
        if let Some(key) = self.map_out_key(key) {
            self.blackboard.insert(key, Rc::new(val));
        }
    }

    pub fn set_any(&mut self, key: impl Into<Symbol>, val: Rc<dyn Any>) {
        if let Some(key) = self.map_out_key(key) {
            self.blackboard.insert(key, val);
        }
    }

    fn map_out_key(&self, key: impl Into<Symbol>) -> Option<Symbol> {
        let key = key.into();
        let mapped = self.blackboard_map.get(&key);
        match mapped {
            None => Some(key),
            Some(BlackboardValue::Ref(mapped, ty)) => {
                if matches!(ty, PortType::Output | PortType::InOut) {
                    Some(*mapped)
                } else {
                    if self.strict {
                        panic!("Port {:?} is not specified as output or inout port", key);
                    }
                    None
                }
            }
            Some(BlackboardValue::Literal(_)) => panic!("Cannot write to a literal!"),
        }
    }

    // pub fn get_env(&mut self) -> Option<&mut E> {
    //     self.env
    // }
}
