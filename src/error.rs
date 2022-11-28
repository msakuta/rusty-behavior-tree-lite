use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum LoadYamlError {
    Yaml(serde_yaml::Error),
    Missing,
    AddChildError(AddChildError),
}

impl Display for LoadYamlError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Yaml(e) => e.fmt(fmt),
            Self::Missing => write!(fmt, "Missing"),
            Self::AddChildError(e) => e.fmt(fmt),
        }
    }
}

impl std::error::Error for LoadYamlError {}

impl From<serde_yaml::Error> for LoadYamlError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AddChildError {
    TooManyNodes,
}

impl Display for AddChildError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::TooManyNodes => write!(fmt, "Attempted to add too many nodes"),
        }
    }
}

pub type AddChildResult = Result<(), AddChildError>;

impl std::error::Error for AddChildError {}

#[derive(Debug)]
#[non_exhaustive]
pub enum LoadError {
    MissingTree,
    MissingNode(String),
    AddChildError(AddChildError, String),
    PortUnmatch { node: String, port: String },
    PortIOUnmatch { node: String, port: String },
    InfiniteRecursion { node: String },
}

impl Display for LoadError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingTree => write!(fmt, "The main tree does not exist"),
            Self::MissingNode(node) => {
                write!(fmt, "Node type or subtree name not found {:?}", node)
            }
            Self::AddChildError(e, node) => {
                e.fmt(fmt)?;
                write!(fmt, " to {}", node)
            }
            Self::PortUnmatch { node, port } => write!(
                fmt,
                "Port {:?} was not provided by the node {:?}",
                port, node
            ),
            Self::PortIOUnmatch { node, port } => write!(
                fmt,
                "Port {:?} on node {:?} has wrong input/output indication",
                port, node
            ),
            Self::InfiniteRecursion { node } => write!(
                fmt,
                "Inifinite recusion detected; the same subtree {node:?} was used in itself"
            ),
        }
    }
}

impl std::error::Error for LoadError {}
