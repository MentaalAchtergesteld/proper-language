use crate::{astbuilder, common::{AssignmentOperator, BinaryOperator, Path, PathSegment, Pattern}, hir};

pub struct Desugarer {
    temp_counter: usize,
}

impl Desugarer {
    pub fn new() -> Self { Desugarer { temp_counter: 0 } }

    pub fn push_temp(&mut self) -> usize {
        let count = self.temp_counter;
        self.temp_counter += 1;

        count
    }

    pub fn desugar_statement(&mut self, stmt: astbuilder::Statement) -> hir::Statement {
        match stmt {
            astbuilder::Statement::Let { name, type_annotation, value } => hir::Statement::Let {
                name,
                type_annotation,
                value: self.desugar_expression(value)
            },
            astbuilder::Statement::FunctionDefinition(f) => hir::Statement::FunctionDefinition(hir::FunctionDefinition {
                signature: f.signature,
                body: self.desugar_expression(f.body),
            }),
            astbuilder::Statement::StructDefinition(s) => hir::Statement::StructDefinition(s),
            astbuilder::Statement::EnumDefinition(e) => hir::Statement::EnumDefinition(e),
            astbuilder::Statement::TraitDefinition(t) => hir::Statement::TraitDefinition(t),
            astbuilder::Statement::ImplBlock(i) => hir::Statement::ImplBlock(hir::ImplBlock {
                trait_path: i.trait_path,
                type_path: i.type_path,
                functions: i.functions.iter().map(|f| self.desugar_statement(f.clone())).collect(),
            }),
            astbuilder::Statement::ExpressionStatement(e) => hir::Statement::Expression(self.desugar_expression(e)),
        }
    }

    pub fn desugar_expression(&mut self, expr: astbuilder::Expression) -> hir::Expression {
        match expr {
            astbuilder::Expression::Literal(val) => hir::Expression::Literal(val),
            astbuilder::Expression::Path(path) => hir::Expression::Path(path),
            astbuilder::Expression::If { condition, then_branch, else_branch } => hir::Expression::If {
                condition: Box::new(self.desugar_expression(*condition)),
                then_branch: Box::new(self.desugar_expression(*then_branch)),
                else_branch: else_branch.map(|e| Box::new(self.desugar_expression(*e))),
            },
            astbuilder::Expression::Match { value, arms } => hir::Expression::Match {
                value: Box::new(self.desugar_expression(*value)),
                arms: arms.iter().map(| astbuilder::MatchArm { pattern, body} | hir::MatchArm {
                    pattern: pattern.clone(),
                    body: self.desugar_expression(body.clone())
                })
                .collect(),
            },
            astbuilder::Expression::Block(statements) => hir::Expression::Block(statements.iter().map(|e| self.desugar_statement(e.clone())).collect()),
            astbuilder::Expression::Closure { params, return_type, body } => hir::Expression::Closure {
                params, return_type, body: Box::new(self.desugar_expression(*body)),
            },
            astbuilder::Expression::Array(elems) => hir::Expression::Array(elems.iter().map(|e| self.desugar_expression(e.clone())).collect()),
            astbuilder::Expression::Tuple(elems) => hir::Expression::Tuple(elems.iter().map(|e| self.desugar_expression(e.clone())).collect()),
            astbuilder::Expression::StructLiteral { path, fields } => hir::Expression::StructLiteral {
                path,
                fields: fields.iter().map(|f| (f.0.clone(), self.desugar_expression(f.1.clone()))).collect()
            },
            astbuilder::Expression::Binary { left, op, right } => hir::Expression::Binary {
                left: Box::new(self.desugar_expression(*left)),
                op,
                right: Box::new(self.desugar_expression(*right))
            },
            astbuilder::Expression::Unary { op, right } => hir::Expression::Unary { op, right: Box::new(self.desugar_expression(*right)) },
            astbuilder::Expression::Call { callee, args } => hir::Expression::Call {
                callee: Box::new(self.desugar_expression(*callee)),
                args: args.iter().map(|e| self.desugar_expression(e.clone())).collect()
            },
            astbuilder::Expression::Index { callee, index } => hir::Expression::Index {
                callee: Box::new(self.desugar_expression(*callee)),
                index: Box::new(self.desugar_expression(*index)),
            },
            astbuilder::Expression::Field { callee, field } => hir::Expression::Field {
                callee: Box::new(self.desugar_expression(*callee)),
                field
            },

            astbuilder::Expression::Return(expr) => hir::Expression::Return(Box::new(self.desugar_expression(*expr))),
            astbuilder::Expression::Break => hir::Expression::Break,
            astbuilder::Expression::Continue => hir::Expression::Continue,

            astbuilder::Expression::IfLet { pattern, value, then_branch, else_branch } => {
                let desugared_value = self.desugar_expression(*value);
                let desugared_then = self.desugar_expression(*then_branch);

                let desugared_else = if let Some(e) = else_branch {
                    self.desugar_expression(*e)
                } else {
                    hir::Expression::Tuple(vec![])
                };

                hir::Expression::Match {
                    value: Box::new(desugared_value),
                    arms: vec![
                        hir::MatchArm { pattern, body: desugared_then},
                        hir::MatchArm { pattern: Pattern::Wildcard, body: desugared_else},
                    ]
                }
            },
            astbuilder::Expression::While { condition, body } => {
                let desugared_condition = self.desugar_expression(*condition);
                let desugared_body = self.desugar_expression(*body);

                let if_check = hir::Expression::If {
                    condition: Box::new(desugared_condition),
                    then_branch: Box::new(desugared_body),
                    else_branch: Some(Box::new(hir::Expression::Break))
                };

                hir::Expression::Loop(Box::new(if_check))
            },
            astbuilder::Expression::WhileLet { pattern, value, body } => {
                let desugared_value = self.desugar_expression(*value);
                let desugared_body = self.desugar_expression(*body);

                let match_expr = hir::Expression::Match {
                    value: Box::new(desugared_value),
                    arms: vec![
                        hir::MatchArm { pattern, body: desugared_body},
                        hir::MatchArm { pattern: Pattern::Wildcard, body: hir::Expression::Break },
                    ]
                };

                hir::Expression::Loop(Box::new(match_expr))
            },
            astbuilder::Expression::For { pattern, iterable, body } => {
                let iter_name = format!("$iter_{}", self.push_temp());

                let let_iter_stmt = hir::Statement::Let {
                    name: iter_name.clone(),
                    type_annotation: None,
                    value: hir::Expression::Call {
                        callee: Box::new(hir::Expression::Path(Path::new()
                            .push(PathSegment::ident("IntoIterator"))
                            .push(PathSegment::ident("into_iter"))
                        )),
                        args: vec![self.desugar_expression(*iterable)],
                    }
                };

                let match_expr = hir::Expression::Match {
                    value: Box::new(hir::Expression::Call {
                        callee: Box::new(hir::Expression::Path(Path::new()
                            .push(PathSegment::ident("Iterator"))
                            .push(PathSegment::ident("next"))
                        )),
                        args: vec![ hir::Expression::Path(Path::new().push(PathSegment::ident(&iter_name))) ]
                    }),
                    arms: vec![
                        hir::MatchArm {
                            pattern: Pattern::Tuple {
                                path: Some(Path::new()
                                    .push(PathSegment::ident("Option"))
                                    .push(PathSegment::ident("Some"))
                                ),
                                patterns: vec![pattern]
                            },
                            body: self.desugar_expression(*body)
                        },
                        hir::MatchArm {
                            pattern: Pattern::Path(Path::new()
                                .push(PathSegment::ident("Option"))
                                .push(PathSegment::ident("None"))
                            ),
                            body: hir::Expression::Break
                        }
                    ]
                };

                let loop_expr = hir::Expression::Loop(Box::new(match_expr));

                hir::Expression::Block(vec![
                    let_iter_stmt,
                    hir::Statement::Expression(loop_expr),
                ])
            },
            astbuilder::Expression::Try(expr) => {
                let desugared = self.desugar_expression(*expr);

                let ok_pattern = Pattern::Tuple {
                    path: Some(Path::new()
                        .push(PathSegment::ident("Result"))
                        .push(PathSegment::ident("Ok"))
                    ),
                    patterns: vec![ Pattern::Identifier("val".to_string()) ]
                };

                let err_pattern = Pattern::Tuple {
                    path: Some(Path::new()
                        .push(PathSegment::ident("Result"))
                        .push(PathSegment::ident("Err"))
                    ),
                    patterns: vec![ Pattern::Identifier("err".to_string()) ]
                };

                let err_var_expr = hir::Expression::Path(Path::new()
                    .push(PathSegment::ident("err"))
                );
                let return_err = hir::Expression::Return(Box::new(hir::Expression::Call {
                    callee: Box::new(hir::Expression::Path(Path::new()
                        .push(PathSegment::ident("Result"))
                        .push(PathSegment::ident("Err"))
                    )),
                    args: vec![err_var_expr]
                }));
                let val_expr = hir::Expression::Path(Path::new().push(PathSegment::ident("val")));
                
                hir::Expression::Match {
                    value: Box::new(desugared),
                    arms: vec![
                        hir::MatchArm { pattern: ok_pattern, body: val_expr },
                        hir::MatchArm { pattern: err_pattern, body: return_err},
                    ]
                }
            }
            astbuilder::Expression::Range { start, end } => {
                let desugared_start = self.desugar_expression(*start);
                let desugared_end = self.desugar_expression(*end);

                let range_path = Path::new()
                    .push(PathSegment::ident("Range"));

                hir::Expression::StructLiteral {
                    path: range_path,
                    fields: vec![
                        ("start".to_string(), desugared_start),
                        ("end".to_string(), desugared_end),
                    ]
                }
            }
            astbuilder::Expression::Assign { left, op, right } => {
                let op = match op {
                    AssignmentOperator::Assign => None,
                    AssignmentOperator::AddAssign => Some(BinaryOperator::Add),
                    AssignmentOperator::SubAssign => Some(BinaryOperator::Subtract),
                    AssignmentOperator::MulAssign => Some(BinaryOperator::Multiply),
                    AssignmentOperator::DivAssign => Some(BinaryOperator::Divide),
                    AssignmentOperator::ModAssign => Some(BinaryOperator::Modulo),
                    AssignmentOperator::AndAssign => Some(BinaryOperator::BitwiseAnd),
                    AssignmentOperator::OrAssign => Some(BinaryOperator::BitwiseOr),
                    AssignmentOperator::XorAssign => Some(BinaryOperator::BitwiseXor),
                    AssignmentOperator::RightShiftAssign => Some(BinaryOperator::RightShift),
                    AssignmentOperator::LeftShiftAssign => Some(BinaryOperator::LeftShift),
                };

                let desugared_left = self.desugar_expression(*left);
                let desugared_right = self.desugar_expression(*right);

                if let Some(op) = op {
                    hir::Expression::Assign {
                        target: Box::new(desugared_left.clone()),
                        value: Box::new(hir::Expression::Binary {
                            left: Box::new(desugared_left),
                            op,
                            right: Box::new(desugared_right),
                        })
                    }
                } else {
                    hir::Expression::Assign { target: Box::new(desugared_left), value: Box::new(desugared_right) }
                }
            }
        }
    }
}
