use super::*;

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
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_ports(
                            "PrintBodyNode",
                            vec![
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
                            ]
                        )
                    )
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
                TreeDef::new_with_child(
                    "Sequence",
                    TreeDef::new_with_ports(
                        "PrintBodyNode",
                        vec![
                            PortMap {
                                ty: PortType::Input,
                                node_port: "in_socket",
                                blackboard_value: BlackboardValue::Literal("in_val".to_string()),
                            },
                            PortMap {
                                ty: PortType::Output,
                                node_port: "out_socket",
                                blackboard_value: BlackboardValue::Ref("out_val"),
                            }
                        ]
                    )
                )
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
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_ports(
                            "PrintBodyNode",
                            vec![
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
                            ]
                        )
                    )
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
                        TreeDef::new_with_child(
                            "Sequence",
                            TreeDef::new_with_ports(
                                "sub",
                                vec![PortMap {
                                    ty: PortType::Input,
                                    node_port: "port",
                                    blackboard_value: BlackboardValue::Ref("input"),
                                }]
                            )
                        )
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
                        root: TreeDef::new_with_child(
                            "Sequence",
                            TreeDef::new_with_ports(
                                "PrintBodyNode",
                                vec![
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
                                ]
                            )
                        )
                    }
                ],
            }
        ))
    );
}

#[test]
fn test_condition() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (ConditionNode) {
        Yes
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new("ConditionNode"),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_condition_with_args() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (ConditionNode (input <- here)) {
        Yes
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new_with_ports(
                                    "ConditionNode",
                                    vec![PortMap {
                                        ty: PortType::Input,
                                        node_port: "input",
                                        blackboard_value: BlackboardValue::Ref("here"),
                                    }]
                                ),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_condition_with_blocks() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (ConditionNode {
        No
    }) {
        Yes
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new_with_child("ConditionNode", TreeDef::new("No"),),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_condition_else() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (ConditionNode) {
        Yes
    } else {
        No
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new("ConditionNode"),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                                TreeDef::new_with_child("Sequence", TreeDef::new("No")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_condition_not() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (!ConditionNode) {
        Yes
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new_with_child("Inverter", TreeDef::new("ConditionNode")),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_condition_not_not() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    if (!!ConditionNode) {
        Yes
    }
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_child(
                        "Sequence",
                        TreeDef::new_with_children(
                            "if",
                            vec![
                                TreeDef::new_with_child(
                                    "Inverter",
                                    TreeDef::new_with_child(
                                        "Inverter",
                                        TreeDef::new("ConditionNode")
                                    )
                                ),
                                TreeDef::new_with_child("Sequence", TreeDef::new("Yes")),
                            ],
                        )
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_var() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = true
}
"
        ),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_children_and_vars(
                        "Sequence",
                        vec![TreeDef::new_with_ports(
                            "SetBool",
                            vec![
                                PortMap {
                                    node_port: "value",
                                    blackboard_value: BlackboardValue::Literal("true".to_owned()),
                                    ty: PortType::Input,
                                },
                                PortMap {
                                    node_port: "output",
                                    blackboard_value: BlackboardValue::Ref("a"),
                                    ty: PortType::Output,
                                }
                            ]
                        )],
                        vec![VarDef {
                            name: "a",
                            init: Some("true"),
                        }],
                    )
                )]
            }
        ))
    );
}
