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
    var a
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
                        vec![],
                        vec![VarDef {
                            name: "a",
                            init: None,
                        }],
                    )
                )]
            }
        ))
    );
}

fn set_bool<'a>(name: &'a str, value: &str) -> TreeDef<'a> {
    TreeDef::new_with_ports(
        "SetBool",
        vec![
            PortMap {
                node_port: "value",
                blackboard_value: BlackboardValue::Literal(value.to_owned()),
                ty: PortType::Input,
            },
            PortMap {
                node_port: "output",
                blackboard_value: BlackboardValue::Ref(name),
                ty: PortType::Output,
            },
        ],
    )
}

#[test]
fn test_var_def() {
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
                        vec![set_bool("a", "true")],
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

#[test]
fn test_line_comment() {
    let source = "
# This is a comment at the top level.
#
tree main = Sequence { # This is a comment after opening brace.
           # This is a comment in a whole line.
    var a  # This is a comment after a variable declaration.
    Yes    # This is a comment after a node.
}          # This is a comment after a closing brace.
";
    assert_eq!(
        parse_file(source),
        Ok((
            "",
            TreeSource {
                node_defs: vec![],
                tree_defs: vec![TreeRootDef::new(
                    "main",
                    TreeDef::new_with_children_and_vars(
                        "Sequence",
                        vec![TreeDef::new("Yes")],
                        vec![VarDef {
                            name: "a",
                            init: None,
                        }],
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_var_cond() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = false
    !a
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
                        vec![
                            set_bool("a", "false"),
                            TreeDef::new_with_child("Inverter", TreeDef::new("a"))
                        ],
                        vec![VarDef {
                            name: "a",
                            init: Some("false"),
                        }],
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_cond_and() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = false
    var b = true
    !a && b
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
                        vec![
                            set_bool("a", "false"),
                            set_bool("b", "true"),
                            TreeDef::new_with_children(
                                "Sequence",
                                vec![
                                    TreeDef::new_with_child("Inverter", TreeDef::new("a")),
                                    TreeDef::new("b")
                                ]
                            ),
                        ],
                        vec![
                            VarDef {
                                name: "a",
                                init: Some("false"),
                            },
                            VarDef {
                                name: "b",
                                init: Some("true"),
                            }
                        ],
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_cond_or() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = false
    var b = true
    a || !b
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
                        vec![
                            set_bool("a", "false"),
                            set_bool("b", "true"),
                            TreeDef::new_with_children(
                                "Fallback",
                                vec![
                                    TreeDef::new("a"),
                                    TreeDef::new_with_child("Inverter", TreeDef::new("b"))
                                ]
                            ),
                        ],
                        vec![
                            VarDef {
                                name: "a",
                                init: Some("false"),
                            },
                            VarDef {
                                name: "b",
                                init: Some("true"),
                            }
                        ],
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_cond_or_and() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = false
    var b = true
    var c = true
    if (!a || b && c) {}
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
                        vec![
                            set_bool("a", "false"),
                            set_bool("b", "true"),
                            set_bool("c", "true"),
                            TreeDef::new_with_children(
                                "if",
                                vec![
                                    TreeDef::new_with_children(
                                        "Fallback",
                                        vec![
                                            TreeDef::new_with_child("Inverter", TreeDef::new("a")),
                                            TreeDef::new_with_children(
                                                "Sequence",
                                                vec![TreeDef::new("b"), TreeDef::new("c")]
                                            )
                                        ]
                                    ),
                                    TreeDef::new("Sequence")
                                ]
                            )
                        ],
                        vec![
                            VarDef {
                                name: "a",
                                init: Some("false"),
                            },
                            VarDef {
                                name: "b",
                                init: Some("true"),
                            },
                            VarDef {
                                name: "c",
                                init: Some("true"),
                            }
                        ],
                    )
                )]
            }
        ))
    );
}

#[test]
fn test_cond_paren() {
    assert_eq!(
        parse_file(
            "
tree main = Sequence {
    var a = false
    var b = true
    var c = true
    if ((!a || b) && c) {}
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
                        vec![
                            set_bool("a", "false"),
                            set_bool("b", "true"),
                            set_bool("c", "true"),
                            TreeDef::new_with_children(
                                "if",
                                vec![
                                    TreeDef::new_with_children(
                                        "Sequence",
                                        vec![
                                            TreeDef::new_with_children(
                                                "Fallback",
                                                vec![
                                                    TreeDef::new_with_child(
                                                        "Inverter",
                                                        TreeDef::new("a")
                                                    ),
                                                    TreeDef::new("b")
                                                ]
                                            ),
                                            TreeDef::new("c")
                                        ]
                                    ),
                                    TreeDef::new("Sequence")
                                ]
                            )
                        ],
                        vec![
                            VarDef {
                                name: "a",
                                init: Some("false"),
                            },
                            VarDef {
                                name: "b",
                                init: Some("true"),
                            },
                            VarDef {
                                name: "c",
                                init: Some("true"),
                            }
                        ],
                    )
                )]
            }
        ))
    );
}
