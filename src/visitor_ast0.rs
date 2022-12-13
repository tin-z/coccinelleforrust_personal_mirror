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

fn visit_path_type<D>(worker: &mut worker<D>, aexpr: syntax::ast::PathType) {
    worker.work_on_node(Box::new(&aexpr), &|worker| {
        aexpr.path().map_or((), |node|{
            visit_path(worker, node);
        });
    });
}

fn visit_path_segment<'a, D>(worker: &mut worker<D>, aexpr: syntax::ast::PathSegment) {
    worker.work_on_node(Box::new(&aexpr), &|worker| {
        worker.work_on_token(aexpr.coloncolon_token()); //https://stackoverflow.com/questions/48864045/why-does-using-optionmap-to-boxnew-a-trait-object-not-work
        match aexpr.name_ref() {
            Some(node) => {
                worker.work_on_node(Box::new(&node), &|worker| {});
            }
            None => {}
        }
        aexpr.path_type().map_or((), |node|{
            visit_path_type(worker, node);
        });
    });
}

fn visit_path<'a, D>(worker: &mut worker<D>, aexpr: syntax::ast::Path) {
    worker.work_on_node(Box::new(&aexpr), &|worker| {
        aexpr.qualifier().map_or((), |node|{
            visit_path(worker, node);
        });
        worker.work_on_token(aexpr.coloncolon_token());

        aexpr.segment().map_or((), |node|{
            visit_path_segment(worker, node);
        });
    });    
}
fn visit_stmts<'a, D>(worker: &mut worker<D>, node: syntax::ast::StmtList) {
    worker.work_on_node(Box::new(&node), &|worker| {
        for stmt in node.statements() {
            match stmt {
                syntax::ast::Stmt::ExprStmt(estmt) => {
                    estmt.expr().map_or((), |node|{
                        visit_expr(worker, node);
                    });
                }
                syntax::ast::Stmt::Item(istmt) => {
                    visit_item(worker, istmt);
                }
                syntax::ast::Stmt::LetStmt(lstmt) => {
                    lstmt.initializer().map_or((), |node|{
                        visit_expr(worker, node);
                    });
                    match lstmt.let_else() {
                        Some(le) => match le.block_expr() {
                            Some(bexpr) => visit_expr(worker, BlockExpr(bexpr)),
                            None => {}
                        },
                        None => {}
                    }
                }
            }
        }
        node.tail_expr().map_or((), |node|{
            visit_expr(worker, node);
        });
    });
}

fn dummydyn(aexpr: Box<dyn AstNode>) {
    aexpr.as_ref();
}

fn visit_expr<'a, D>(worker: &mut worker<D>, node: syntax::ast::Expr) {
    match node {
        ArrayExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                for ex in aexpr.exprs() {
                    visit_expr(worker, ex);
                }
            });
        }
        AwaitExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BinExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.lhs().map_or((), |node|{
                    visit_expr(worker, node);
                });
                aexpr.rhs().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BoxExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BreakExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        CallExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ClosureExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.param_list().map_or((), |node| {
                    visit_param_list(worker, node);
                });
                aexpr.body().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        CastExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ContinueExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        FieldExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ForExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.iterable().map_or((), |node|{
                    visit_expr(worker, node);
                });
                match aexpr.loop_body() {
                    Some(bexpr) => {
                        visit_expr(worker, BlockExpr(bexpr));
                    }
                    None => {}
                }
            });
        }
        IfExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.condition().map_or((), |node|{
                    visit_expr(worker, node);
                });
                aexpr.then_branch().map_or((), |branch| {
                    visit_expr(worker, BlockExpr(branch));
                });
                match aexpr.else_branch() {
                    Some(syntax::ast::ElseBranch::Block(block)) => {
                        visit_expr(worker, BlockExpr(block));
                    }
                    Some(syntax::ast::ElseBranch::IfExpr(ifexpr)) => {
                        visit_expr(worker, IfExpr(ifexpr));
                    }
                    None => {}
                }
            });
        }
        IndexExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        Literal(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        LoopExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            //TODO
        }
        MacroExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            /*TODO*/
        }
        MatchExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
            aexpr.expr().map_or((), |node|{
                visit_expr(worker, node);
            });
        }
        MethodCallExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.receiver().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.dot_token());
                aexpr.name_ref().map_or((), |node|{
                    worker.work_on_node(Box::new(&node), &|worker| {});
                }) 
            });
        }
        ParenExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        PathExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.path().map_or((), |node|{
                    visit_path(worker, node);
                });
            });
        }
        PrefixExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        RangeExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| ());
        }
        RecordExpr(aexpr) => {
            worker.work_on_node(
                Box::new(&aexpr),
                &|worker| match aexpr.record_expr_field_list() {
                    Some(al) => {
                        al.spread().map_or((), |node|{
                            visit_expr(worker, node);
                        });
                    }
                    None => {}
                },
            );
        }
        RefExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ReturnExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
       TryExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        TupleExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                for child in aexpr.fields() {
                    visit_expr(worker, child);
                }
            });
        }
        WhileExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
        }
        YieldExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BlockExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.stmt_list().map_or((), |node| {
                    visit_stmts(worker, node);
                });
            });
        }
        LetExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        UnderscoreExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &|worker| {});
        }
    }
}

fn visit_param_list<'a, D>(worker: &mut worker<D>, plist: syntax::ast::ParamList) {

    worker.work_on_node(Box::new(&plist), &|worker| {
    worker.work_on_token(plist.l_paren_token());
    worker.work_on_token(plist.comma_token());
    for param in plist.params() {
        //wrap_pat(lindex, param.pat());
        worker.work_on_token(param.colon_token());
        param.ty().map_or((), |node| {
            visit_type(worker, node);
        });
        worker.work_on_token(param.dotdotdot_token());
    }
    worker.work_on_token(plist.r_paren_token());
    worker.work_on_token(plist.pipe_token());
    });

}

fn visit_name<'a, D>(worker: &mut worker<D>, node: syntax::ast::Name) {

    worker.work_on_node(Box::new(&node), &|worker| {
        worker.work_on_token(node.ident_token());
        worker.work_on_token(node.self_token());
    });
}

fn visit_type<'a, D>(worker: &mut worker<D>, node: syntax::ast::Type) {

    worker.work_on_node(Box::new(&node), &|worker| {});
    //need to work on the other types TODO

}

fn visit_abi<'a, D>(worker: &mut worker<D>, node: syntax::ast::Abi) {

    worker.work_on_node(Box::new(&node), &|worker| {});
    //need to work TODO

}

fn visit_ret_type<'a, D>(worker: &mut worker<D>, node: syntax::ast::RetType) {

    worker.work_on_node(Box::new(&node), &|worker| {});
    //need to work TODO

}

fn visit_item<'a, D>(worker: &mut worker<D>, node: syntax::ast::Item) {
    //notcomplete
    match node {
        syntax::ast::Item::Const(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                node.name().map_or((),
                |node| visit_name(worker, node));
                worker.work_on_token(node.default_token());
                worker.work_on_token(node.const_token());
                worker.work_on_token(node.underscore_token());
                worker.work_on_token(node.colon_token());
                node.ty().map_or((), |node|{
                    visit_type(worker, node);
                });
                worker.work_on_token(node.eq_token());
                node.body().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(node.semicolon_token());
            });
        }
        syntax::ast::Item::Fn(node) => {
            worker.work_on_node(Box::new(&node), &|worker| {
                node.name().map_or((),
                |node| visit_name(worker, node));
                worker.work_on_token(node.default_token());
                worker.work_on_token(node.const_token());
                worker.work_on_token(node.async_token());
                worker.work_on_token(node.unsafe_token());
                node.abi().map_or((), |node| {
                    visit_abi(worker, node);
                });
                worker.work_on_token(node.fn_token());
                node.param_list().map_or((), |node| {
                    visit_param_list(worker, node);
                });
                node.ret_type().map_or((), 
                |node| visit_ret_type(worker, node));
                node.body().map_or((), |node|{
                    visit_expr(worker, BlockExpr(node));
                });
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
}
