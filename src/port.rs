use crate::Symbol;

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
