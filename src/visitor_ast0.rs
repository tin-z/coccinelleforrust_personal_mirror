mod ast0;

use ast0::{info, position_info, wrap};
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use std::vec;
use syntax;
use syntax::ast::Expr::*;
use syntax::ast::ForExpr;
use syntax::ast::HasLoopBody;
use syntax::ast::HasName;
use syntax::ast::{AnyHasArgList, AstNode, HasModuleItem, SourceFile, Type};
use syntax::SyntaxToken;

use self::ast0::bef_aft;

use self::ast0::worker;
pub use self::ast0::Rnode;
pub use self::ast0::Syntax;
pub use self::ast0::Syntax::{Node, Token};

pub fn wrap_keyword_aux<'a>(lindex: &LineIndex, aexpr: Option<SyntaxToken>) -> Option<Rnode> {
    match aexpr {
        Some(aexpr) => {
            let sindex: LineCol = lindex.line_col(aexpr.text_range().start());
            let eindex: LineCol = lindex.line_col(aexpr.text_range().end());

            let pos_info: position_info = position_info::new(
                sindex.line,
                eindex.line,
                0,
                0,
                sindex.col,
                aexpr.text_range().start().into(),
            );
            let info = info::new(
                pos_info,
                false,
                false,
                vec![],
                vec![],
                vec![],
                vec![],
                false,
            );
            let kind = aexpr.kind();
            let wrap: wrap = wrap::new(
                info,
                0,
                ast0::mcodekind::MIXED(),
                None,
                bef_aft {},
                AnyHasArgList::can_cast(kind),
                false,
                false,
                vec![],
            );

            Some(Rnode {
                wrapper: wrap,
                astnode: Token(aexpr),
                children: vec![],
            })
        }
        None => None,
    }
}

pub fn wrap_node_aux<'a>(
    lindex: &LineIndex,
    aexpr: Box<&dyn AstNode>,
    df: &dyn FnMut(&mut worker<Rnode<'a>>),
) -> Option<Rnode<'a>> {
    let sindex: LineCol = lindex.line_col(aexpr.syntax().text_range().start());
    let eindex: LineCol = lindex.line_col(aexpr.syntax().text_range().end());

    let pos_info: position_info = position_info::new(
        sindex.line,
        eindex.line,
        0,
        0,
        sindex.col,
        aexpr.syntax().text_range().start().into(),
    );

    let info = info::new(
        pos_info,
        false,
        false,
        vec![],
        vec![],
        vec![],
        vec![],
        false,
    );
    let wrap: wrap = wrap::new(
        info,
        0,
        ast0::mcodekind::MIXED(),
        Type::cast(aexpr.syntax().to_owned()),
        bef_aft {},
        AnyHasArgList::can_cast(aexpr.syntax().kind()),
        false,
        false,
        vec![],
    );
    Some(Rnode {
        wrapper: wrap,
        astnode: Node(aexpr.syntax().clone()),
        children: vec![],
    })
}

fn visit_path_type<D>(worker: &mut worker<D>, aexpr: Option<syntax::ast::PathType>) {
    match aexpr {
        Some(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_path(worker, aexpr.path());
            });
        }
        None => {}
    }
}

fn visit_path_segment<'a, D>(worker: &mut worker<D>, aexpr: Option<syntax::ast::PathSegment>) {
    match aexpr {
        Some(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                worker.work_on_token(aexpr.coloncolon_token()); //https://stackoverflow.com/questions/48864045/why-does-using-optionmap-to-boxnew-a-trait-object-not-work
                match aexpr.name_ref() {
                    Some(node) => {
                        worker.work_on_node(Box::new(&node), &|worker| {});
                    }
                    None => {}
                }
                visit_path_type(worker, aexpr.path_type());
            });
        }
        None => {}
    }
}

fn visit_path<'a, D>(worker: &mut worker<D>, aexpr: Option<syntax::ast::Path>) {
    match aexpr {
        Some(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_path(worker, aexpr.qualifier());
                worker.work_on_token(aexpr.coloncolon_token());
                visit_path_segment(worker, aexpr.segment());
            });
        }
        None => {}
    }
}
fn visit_stmts<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::StmtList>) {
    match node {
        Some(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                for stmt in node.statements() {
                    match stmt {
                        syntax::ast::Stmt::ExprStmt(estmt) => {
                            visit_expr(worker, estmt.expr());
                        }
                        syntax::ast::Stmt::Item(istmt) => {
                            visit_item(worker, istmt);
                        }
                        syntax::ast::Stmt::LetStmt(lstmt) => {
                            visit_expr(worker, lstmt.initializer());
                            match lstmt.let_else() {
                                Some(le) => match le.block_expr() {
                                    Some(bexpr) => visit_expr(worker, Some(BlockExpr(bexpr))),
                                    None => {}
                                },
                                None => {}
                            }
                        }
                    }
                }
                visit_expr(worker, node.tail_expr());
            });
        }
        None => {}
    }
}

fn dummydyn(aexpr: Box<dyn AstNode>) {
    aexpr.as_ref();
}

fn visit_expr<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::Expr>) {
    match node {
        Some(ArrayExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                for ex in aexpr.exprs() {
                    visit_expr(worker, Some(ex));
                }
            });
        }
        Some(AwaitExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(BinExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.lhs());
                visit_expr(worker, aexpr.rhs());
            });
        }
        Some(BoxExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(BreakExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(CallExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(ClosureExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_param_list(worker, aexpr.param_list());
                visit_expr(worker, aexpr.body());
            });
        }
        Some(CastExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(ContinueExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        Some(FieldExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(ForExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.iterable());
                match aexpr.loop_body() {
                    Some(bexpr) => {
                        visit_expr(worker, Some(BlockExpr(bexpr)));
                    }
                    None => {}
                }
            });
        }
        Some(IfExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.condition());
                match aexpr.then_branch() {
                    Some(branch) => {
                        visit_expr(worker, Some(BlockExpr(branch)));
                    }
                    None => {}
                }
                match aexpr.else_branch() {
                    Some(syntax::ast::ElseBranch::Block(block)) => {
                        visit_expr(worker, Some(BlockExpr(block)));
                    }
                    Some(syntax::ast::ElseBranch::IfExpr(ifexpr)) => {
                        visit_expr(worker, Some(IfExpr(ifexpr)));
                    }
                    None => {}
                }
            });
        }
        Some(IndexExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        Some(Literal(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        Some(LoopExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        Some(MacroExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            /*TODO*/
        }
        Some(MatchExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            visit_expr(worker, aexpr.expr());
        }
        Some(MethodCallExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.receiver());
                worker.work_on_token(aexpr.dot_token());
                match aexpr.name_ref() {
                    Some(node) => {
                        worker.work_on_node(Box::new(&node), &|worker| {});
                    }
                    None => {}
                }
            });
        }
        Some(ParenExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(PathExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_path(worker, aexpr.path());
            });
        }
        Some(PrefixExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(RangeExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| ());
        }
        Some(RecordExpr(aexpr)) => {
            worker.work_on_node(
                Box::new(&aexpr),
                &|worker| match aexpr.record_expr_field_list() {
                    Some(al) => {
                        visit_expr(worker, al.spread());
                    }
                    None => {}
                },
            );
        }
        Some(RefExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(ReturnExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(TryExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(TupleExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                for child in aexpr.fields() {
                    visit_expr(worker, Some(child));
                }
            });
        }
        Some(WhileExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
        }
        Some(YieldExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(BlockExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_stmts(worker, aexpr.stmt_list());
            });
        }
        Some(LetExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                visit_expr(worker, aexpr.expr());
            });
        }
        Some(UnderscoreExpr(aexpr)) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
        }
        None => {}
    }
}

fn visit_param_list<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::ParamList>) {
    match node {
        Some(plist) => {
                worker.work_on_node(Box::new(&plist), &|worker| {
                worker.work_on_token(plist.l_paren_token());
                worker.work_on_token(plist.comma_token());
                for param in plist.params() {
                    //wrap_pat(lindex, param.pat());
                    worker.work_on_token(param.colon_token());
                    visit_type(worker, param.ty());
                    worker.work_on_token(param.dotdotdot_token());
                }
                worker.work_on_token(plist.r_paren_token());
                worker.work_on_token(plist.pipe_token());
            });
        }
        None => {}
    }
}

fn visit_name<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::Name>) {
    match node {
        Some(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                worker.work_on_token(node.ident_token());
                worker.work_on_token(node.self_token());
            });
        }
        None => {}
    }
}

fn visit_type<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::Type>) {
    match node {
        Some(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {});
            //need to work on the other types TODO
        }
        None => {}
    }
}

fn visit_abi<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::Abi>) {
    match node {
        Some(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {});
            //need to work TODO
        }
        None => {}
    }
}

fn visit_ret_type<'a, D>(worker: &mut worker<D>, node: Option<syntax::ast::RetType>) {
    match node {
        Some(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {});
            //need to work TODO
        }
        None => {}
    }
}

fn visit_item<'a, D>(worker: &mut worker<D>, node: syntax::ast::Item) {
    //notcomplete
    match node {
        syntax::ast::Item::Const(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                visit_name(worker, node.name()); // for each visit worker will keep track of the ast and children
                worker.work_on_token(node.default_token());
                worker.work_on_token(node.const_token());
                worker.work_on_token(node.underscore_token());
                worker.work_on_token(node.colon_token());
                visit_type(worker, node.ty());
                worker.work_on_token(node.eq_token());
                visit_expr(worker, node.body());
                worker.work_on_token(node.semicolon_token());
            });
        }
        syntax::ast::Item::Fn(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                visit_name(worker, node.name()); // for each visit worker will keep track of the ast and children
                worker.work_on_token(node.default_token());
                worker.work_on_token(node.const_token());
                worker.work_on_token(node.async_token());
                worker.work_on_token(node.unsafe_token());
                visit_abi(worker, node.abi());
                worker.work_on_token(node.fn_token());
                visit_param_list(worker, node.param_list());
                visit_ret_type(worker, node.ret_type());
                visit_expr(
                    worker,
                    match node.body() {
                        Some(body) => Some(BlockExpr(body)),
                        None => None,
                    },
                );
                worker.work_on_token(node.semicolon_token());
            });
        }
        syntax::ast::Item::Impl(node) => worker.work_on_node(Box::new(&node), &|worker| {
            worker.work_on_token(node.default_token());
            worker.work_on_token(node.unsafe_token());
            worker.work_on_token(node.impl_token());
            worker.work_on_token(node.const_token());
            worker.work_on_token(node.excl_token());
            worker.work_on_token(node.for_token());
            match node.assoc_item_list() {
                Some(item) => {
                    worker.work_on_token(item.l_curly_token());
                    for item in item.assoc_items() {
                        match item {
                            syntax::ast::AssocItem::Const(cnt) => {
                                visit_item(worker, syntax::ast::Item::Const(cnt));
                            }
                            syntax::ast::AssocItem::Fn(f) => {
                                visit_item(worker, syntax::ast::Item::Fn(f));
                            }
                            syntax::ast::AssocItem::MacroCall(mc) => {
                                visit_item(worker, syntax::ast::Item::MacroCall(mc));
                            }
                            syntax::ast::AssocItem::TypeAlias(ta) => {
                                visit_item(worker, syntax::ast::Item::TypeAlias(ta));
                            }
                        }
                    }
                    worker.work_on_token(item.r_curly_token());
                }
                None => {}
            }
        }),
        _ => {}
    }
}

pub fn wraproot(contents: &str) {
    let root = SourceFile::parse(contents).tree();
    let items = root.items();
    let lindex: LineIndex = LineIndex::new(&root.to_string()[..]);

    let mut worker = worker::new(&lindex, wrap_node_aux, wrap_keyword_aux);

    for item in items.into_iter() {
        //for now skips Attributes
        //visit(worker, item)
        visit_item(&mut worker, item);
    }

    let sindex: LineCol = lindex.line_col(root.syntax().text_range().start());
    let eindex: LineCol = lindex.line_col(root.syntax().text_range().end());

    let pos_info: position_info = position_info {
        line_start: sindex.line,
        line_end: eindex.line,
        logical_start: 0, //TODO
        logical_end: 0,
        column: sindex.col,
        offset: root.syntax().text_range().start().into(),
    };
    let info = info::new(
        pos_info,
        false,
        false,
        vec![],
        vec![],
        vec![],
        vec![],
        false,
    );
    let wrap: wrap = wrap::new(
        info,
        0,
        ast0::mcodekind::MIXED(),
        Type::cast(root.syntax().to_owned()),
        bef_aft {},
        AnyHasArgList::can_cast(root.syntax().kind()),
        false,
        false,
        vec![],
    );
}
