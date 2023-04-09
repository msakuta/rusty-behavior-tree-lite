use crate::{Symbol, parser::{PortMap, BlackboardValue, PortMapOwned}, Blackboard};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PortType {
    Input,
    Output,
    InOut,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PortSpec {
    pub ty: PortType,
    pub key: Symbol,
}

impl PortSpec {
    pub fn new_in(key: impl Into<Symbol>) -> Self {
        Self {
            ty: PortType::Input,
            key: key.into(),
        }
    }

    pub fn new_out(key: impl Into<Symbol>) -> Self {
        Self {
            ty: PortType::Output,
            key: key.into(),
        }
    }

    pub fn new_inout(key: impl Into<Symbol>) -> Self {
        Self {
            ty: PortType::InOut,
            key: key.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlackboardValueOwned {
    /// Literal value could have been decoded, so it is an owned string.
    Literal(String),
    Ref(String),
}

impl crate::BlackboardValue {
    pub fn to_owned2(&self) -> BlackboardValueOwned {
        match self {
            Self::Literal(s) => BlackboardValueOwned::Literal(s.clone()),
            Self::Ref(s, _) => BlackboardValueOwned::Ref(s.to_string())
        }
    }
}

pub trait AbstractPortMap<'src> {
    fn get_type(&self) -> PortType;
    fn node_port(&self) -> &str;
    fn blackboard_value(&self) -> BlackboardValueOwned;
}

impl<'src> AbstractPortMap<'src> for PortMap<'src> {
    fn get_type(&self) -> PortType {
        self.ty
    }

    fn node_port(&self) -> &'src str {
        self.node_port
    }

    fn blackboard_value(&self) -> BlackboardValueOwned {
        match &self.blackboard_value {
            BlackboardValue::Literal(s) => BlackboardValueOwned::Literal(s.clone()),
            BlackboardValue::Ref(s) => BlackboardValueOwned::Ref(s.to_string()),
        }
    }
}

impl<'src> AbstractPortMap<'src> for PortMapOwned {
    fn get_type(&self) -> PortType {
        self.ty
    }

    fn node_port(&self) -> &str {
        &self.node_port
    }

    fn blackboard_value(&self) -> BlackboardValueOwned {
        self.blackboard_value.clone()
    }
}