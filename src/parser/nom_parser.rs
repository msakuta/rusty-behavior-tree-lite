use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0, none_of, one_of, space0},
    combinator::{opt, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use crate::PortType;

#[derive(Debug, PartialEq)]
pub struct NodeDef<'src> {
    name: &'src str,
    ports: Vec<PortDef<'src>>,
}

impl<'src> NodeDef<'src> {
    pub fn new(name: &'src str) -> Self {
        Self {
            name,
            ports: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct PortDef<'src> {
    pub direction: PortType,
    pub name: &'src str,
    pub ty: Option<&'src str>,
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn newlines(i: &str) -> IResult<&str, ()> {
    delimited(space0, many1(one_of("\r\n")), space0)(i).map(|(rest, _)| (rest, ()))
}

fn port_def<'src>(i: &'src str) -> IResult<&'src str, PortDef<'src>> {
    let (i, inout) = delimited(space0, alt((tag("in"), tag("out"), tag("inout"))), space0)(i)?;
    let (i, name) = identifier(i)?;
    let (i, ty) = opt(preceded(delimited(space0, char(':'), space0), identifier))(i)?;
    let (i, _) = multispace0(i)?;
    let direction = match inout {
        "in" => PortType::Input,
        "out" => PortType::Output,
        "inout" => PortType::InOut,
        _ => {
            return Err(nom::Err::Failure(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Verify,
            )))
        }
    };
    Ok((
        i,
        PortDef {
            direction,
            name,
            ty,
        },
    ))
}

fn ports_def<'src>(i: &'src str) -> IResult<&'src str, Vec<PortDef<'src>>> {
    let (i, _) = many0(newlines)(i)?;

    let (i, v) = many0(delimited(space0, port_def, many0(pair(space0, newlines))))(i)?;

    let (i, _) = many0(newlines)(i)?;

    Ok((i, v))
}

pub fn node_def<'src>(i: &'src str) -> IResult<&'src str, NodeDef<'src>> {
    let (i, _) = delimited(multispace0, tag("node"), space0)(i)?;

    let (i, name) = delimited(space0, alphanumeric1, space0)(i)?;

    let (i, ports) = delimited(
        delimited(space0, char('{'), space0),
        ports_def,
        delimited(space0, char('}'), space0),
    )(i)?;

    Ok((i, NodeDef { name, ports }))
}

pub fn parse_nodes<'src>(i: &'src str) -> IResult<&'src str, Vec<NodeDef<'src>>> {
    many0(node_def)(i)
}

#[derive(Debug, PartialEq, Eq)]
pub struct TreeDef<'src> {
    pub(crate) ty: &'src str,
    pub(crate) port_maps: Vec<PortMap<'src>>,
    pub(crate) children: Vec<TreeDef<'src>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlackboardValue<'src> {
    /// Litral value could have decoded, so it is an owned string.
    Literal(String),
    Ref(&'src str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PortMap<'src> {
    pub(crate) ty: PortType,
    pub(crate) node_port: &'src str,
    pub(crate) blackboard_value: BlackboardValue<'src>,
}

fn subtree_ports_def<'src>(i: &'src str) -> IResult<&'src str, Vec<PortDef<'src>>> {
    let (i, ports) = delimited(
        delimited(space0, char('('), space0),
        many0(delimited(space0, port_def, opt(char(',')))),
        delimited(space0, char(')'), space0),
    )(i)?;
    Ok((i, ports))
}

#[derive(Debug, PartialEq)]
pub struct TreeRootDef<'src> {
    pub(crate) name: &'src str,
    pub(crate) root: TreeDef<'src>,
    pub(crate) ports: Vec<PortDef<'src>>,
}

fn parse_tree(i: &str) -> IResult<&str, TreeRootDef> {
    let (i, _) = delimited(multispace0, tag("tree"), space0)(i)?;

    let (i, name) = delimited(space0, identifier, space0)(i)?;

    let (i, ports) = opt(subtree_ports_def)(i)?;

    let (i, _) = delimited(space0, tag("="), space0)(i)?;

    let (i, root) = parse_tree_node(i)?;

    Ok((
        i,
        TreeRootDef {
            name,
            root,
            ports: ports.unwrap_or_else(|| vec![]),
        },
    ))
}

fn tree_children(i: &str) -> IResult<&str, Vec<TreeDef>> {
    let (i, _) = many0(newlines)(i)?;

    let (i, v) = many0(delimited(
        space0,
        parse_tree_node,
        many0(pair(space0, newlines)),
    ))(i)?;

    let (i, _) = many0(newlines)(i)?;

    Ok((i, v))
}

fn parse_tree_node(i: &str) -> IResult<&str, TreeDef> {
    let (i, ty) = delimited(space0, identifier, space0)(i)?;

    let (i, input_ports) = opt(delimited(
        delimited(space0, char('('), space0),
        port_maps,
        delimited(space0, char(')'), space0),
    ))(i)?;

    let (i, children) = opt(delimited(
        delimited(space0, char('{'), space0),
        tree_children,
        delimited(space0, char('}'), space0),
    ))(i)?;

    Ok((
        i,
        TreeDef {
            ty,
            port_maps: input_ports.unwrap_or(vec![]),
            children: children.unwrap_or(vec![]),
        },
    ))
}

fn port_maps(i: &str) -> IResult<&str, Vec<PortMap>> {
    many0(delimited(
        multispace0,
        port_map,
        many0(pair(multispace0, char(','))),
    ))(i)
}

fn port_map(i: &str) -> IResult<&str, PortMap> {
    let (i, node_port) = delimited(space0, identifier, space0)(i)?;

    let (i, inout) = delimited(space0, alt((tag("<->"), tag("<-"), tag("->"))), space0)(i)?;

    let (i, blackboard_name) = delimited(space0, alt((bb_ref, str_literal)), space0)(i)?;

    let ty = match inout {
        "<-" => PortType::Input,
        "->" => PortType::Output,
        "<->" => PortType::InOut,
        _ => {
            return Err(nom::Err::Failure(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Alt,
            )))
        }
    };

    // You cannot output to a literal! It is a parse error rather than runtime error.
    if let BlackboardValue::Literal(_) = blackboard_name {
        if !matches!(ty, PortType::Input) {
            return Err(nom::Err::Failure(nom::error::Error::new(
                i,
                nom::error::ErrorKind::Verify,
            )));
        }
    }

    Ok((
        i,
        PortMap {
            ty,
            node_port,
            blackboard_value: blackboard_name,
        },
    ))
}

fn bb_ref(i: &str) -> IResult<&str, BlackboardValue> {
    let (i, s) = identifier(i)?;
    Ok((i, BlackboardValue::Ref(s)))
}

fn str_literal(input: &str) -> IResult<&str, BlackboardValue> {
    let (r, val) = delimited(
        preceded(multispace0, char('\"')),
        many0(none_of("\"")),
        terminated(char('"'), multispace0),
    )(input)?;
    Ok((
        r,
        BlackboardValue::Literal(
            val.iter()
                .collect::<String>()
                .replace("\\\\", "\\")
                .replace("\\n", "\n"),
        ),
    ))
}

pub fn parse_file(i: &str) -> IResult<&str, TreeSource> {
    enum NodeOrTree<'src> {
        Node(NodeDef<'src>),
        Tree(TreeRootDef<'src>),
    }

    let (i, stmts) = many0(alt((
        |i| {
            let (i, node) = node_def(i)?;
            Ok((i, NodeOrTree::Node(node)))
        },
        |i| {
            let (i, tree) = parse_tree(i)?;
            Ok((i, NodeOrTree::Tree(tree)))
        },
    )))(i)?;

    // Eat up trailing newlines to indicate that the input was thoroughly consumed
    let (i, _) = multispace0(i)?;

    let (node_defs, tree_defs) = stmts.into_iter().fold((vec![], vec![]), |mut acc, cur| {
        match cur {
            NodeOrTree::Node(node) => acc.0.push(node),
            NodeOrTree::Tree(tree) => acc.1.push(tree),
        }
        acc
    });

    Ok((
        i,
        TreeSource {
            node_defs,
            tree_defs,
        },
    ))
}

#[derive(Debug, PartialEq)]
pub struct TreeSource<'src> {
    pub node_defs: Vec<NodeDef<'src>>,
    pub tree_defs: Vec<TreeRootDef<'src>>,
}

#[cfg(test)]
mod test {
    use super::*;

    impl<'src> TreeDef<'src> {
        fn new(ty: &'src str) -> Self {
            Self {
                ty,
                port_maps: vec![],
                children: vec![],
            }
        }

        fn new_with_child(ty: &'src str, child: TreeDef<'src>) -> Self {
            Self {
                ty,
                port_maps: vec![],
                children: vec![child],
            }
        }
    }

    impl<'src> TreeRootDef<'src> {
        fn new(name: &'src str, root: TreeDef<'src>) -> Self {
            Self {
                name,
                root,
                ports: vec![],
            }
        }
    }

    #[test]
    fn test_nodes() {
        assert_eq!(
            parse_nodes(
                "node A {
        }"
            ),
            Ok((
                "",
                vec![NodeDef {
                    name: "A",
                    ports: vec![],
                }]
            ))
        );

        assert_eq!(
            parse_nodes(
                "node A {
            in A: Arm
            out B: Body
        }"
            ),
            Ok((
                "",
                vec![NodeDef {
                    name: "A",
                    ports: vec![
                        PortDef {
                            direction: PortType::Input,
                            name: "A",
                            ty: Some("Arm"),
                        },
                        PortDef {
                            direction: PortType::Output,
                            name: "B",
                            ty: Some("Body"),
                        }
                    ],
                }]
            ))
        );
    }

    #[test]
    fn test_trees() {
        assert_eq!(
            parse_tree(
                "tree main = Sequence {
        }"
            ),
            Ok(("", TreeRootDef::new("main", TreeDef::new("Sequence"))))
        );

        assert_eq!(
            parse_tree(
                "tree main = Sequence {
                    PrintBodyNode
        }"
            ),
            Ok((
                "",
                TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child("Sequence", TreeDef::new("PrintBodyNode"))
                )
            ))
        );
    }

    #[test]
    fn test_tree_ports() {
        assert_eq!(
            parse_tree(
                "tree main = Sequence {
                PrintBodyNode(in_socket <- in_val, out_socket -> out_val, inout_socket <-> inout_val)
    }"
            ),
            Ok((
                "",
                TreeRootDef::new("main",
                    TreeDef {
                        ty: "Sequence",
                        port_maps: vec![],
                        children: vec![TreeDef {
                            ty: "PrintBodyNode",
                            port_maps: vec![
                                PortMap {
                                    ty: PortType::Input,
                                    node_port: "in_socket",
                                    blackboard_value: BlackboardValue::Ref("in_val"),
                                },
                                PortMap {
                                    ty: PortType::Output,
                                    node_port: "out_socket",
                                    blackboard_value: BlackboardValue::Ref("out_val"),
                                },
                                PortMap {
                                    ty: PortType::InOut,
                                    node_port: "inout_socket",
                                    blackboard_value: BlackboardValue::Ref("inout_val"),
                                }
                            ],
                            children: vec![]
                        }]
                    }
                )
            ))
        );
    }

    #[test]
    fn test_port_literal() {
        assert_eq!(
            parse_tree(
                r#"tree main = Sequence {
                PrintBodyNode(in_socket <- "in_val", out_socket -> out_val)
    }"#
            ),
            Ok((
                "",
                TreeRootDef::new(
                    "main",
                    TreeDef {
                        ty: "Sequence",
                        port_maps: vec![],
                        children: vec![TreeDef {
                            ty: "PrintBodyNode",
                            port_maps: vec![
                                PortMap {
                                    ty: PortType::Input,
                                    node_port: "in_socket",
                                    blackboard_value: BlackboardValue::Literal(
                                        "in_val".to_string()
                                    ),
                                },
                                PortMap {
                                    ty: PortType::Output,
                                    node_port: "out_socket",
                                    blackboard_value: BlackboardValue::Ref("out_val"),
                                }
                            ],
                            children: vec![]
                        }]
                    }
                )
            ))
        );
    }

    #[test]
    fn test_file() {
        assert_eq!(
            parse_file(
                "node A {
            in A: Arm
            out B: Body
        }
        tree main = Sequence {
            PrintBodyNode(in_socket <- in_val, out_socket -> out_val)
        }"
            ),
            Ok((
                "",
                TreeSource {
                    node_defs: vec![NodeDef {
                        name: "A",
                        ports: vec![
                            PortDef {
                                direction: PortType::Input,
                                name: "A",
                                ty: Some("Arm"),
                            },
                            PortDef {
                                direction: PortType::Output,
                                name: "B",
                                ty: Some("Body"),
                            }
                        ],
                    }],
                    tree_defs: vec![TreeRootDef::new(
                        "main",
                        TreeDef {
                            ty: "Sequence",
                            port_maps: vec![],
                            children: vec![TreeDef {
                                ty: "PrintBodyNode",
                                port_maps: vec![
                                    PortMap {
                                        ty: PortType::Input,
                                        node_port: "in_socket",
                                        blackboard_value: BlackboardValue::Ref("in_val"),
                                    },
                                    PortMap {
                                        ty: PortType::Output,
                                        node_port: "out_socket",
                                        blackboard_value: BlackboardValue::Ref("out_val"),
                                    }
                                ],
                                children: vec![],
                            }]
                        }
                    )],
                }
            ))
        );
    }

    #[test]
    fn test_subtree() {
        assert_eq!(
            parse_file(
                "
tree main = Sequence {
    sub(port <- input)
}

tree sub(in port, out result) = Sequence {
    PrintBodyNode(in_socket <- in_val, out_socket -> out_val)
}
"
            ),
            Ok((
                "",
                TreeSource {
                    node_defs: vec![],
                    tree_defs: vec![
                        TreeRootDef::new(
                            "main",
                            TreeDef {
                                ty: "Sequence",
                                port_maps: vec![],
                                children: vec![TreeDef {
                                    ty: "sub",
                                    port_maps: vec![PortMap {
                                        ty: PortType::Input,
                                        node_port: "port",
                                        blackboard_value: BlackboardValue::Ref("input"),
                                    }],
                                    children: vec![],
                                }]
                            }
                        ),
                        TreeRootDef {
                            name: "sub",
                            ports: vec![
                                PortDef {
                                    direction: PortType::Input,
                                    name: "port",
                                    ty: None,
                                },
                                PortDef {
                                    direction: PortType::Output,
                                    name: "result",
                                    ty: None,
                                }
                            ],
                            root: TreeDef {
                                ty: "Sequence",
                                port_maps: vec![],
                                children: vec![TreeDef {
                                    ty: "PrintBodyNode",
                                    port_maps: vec![
                                        PortMap {
                                            ty: PortType::Input,
                                            node_port: "in_socket",
                                            blackboard_value: BlackboardValue::Ref("in_val"),
                                        },
                                        PortMap {
                                            ty: PortType::Output,
                                            node_port: "out_socket",
                                            blackboard_value: BlackboardValue::Ref("out_val"),
                                        }
                                    ],
                                    children: vec![],
                                }]
                            }
                        }
                    ],
                }
            ))
        );
    }
}
