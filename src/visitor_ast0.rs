/// visitor_ast0.rs
/// This file contains code for going over(visitng) the Abstract Syntax Tree
/// and output a vector of a user-defined data structure after computation

pub mod ast0;

use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use std::vec;
use syntax;
use syntax::ast::Expr::*;
use syntax::ast::HasLoopBody;
use syntax::ast::HasName;
use syntax::ast::{AnyHasArgList, AstNode, HasModuleItem, SourceFile, Type};
use syntax::SyntaxToken;

use self::ast0::worker;

fn visit_path_type<D>(worker: &mut worker<D>, aexpr: syntax::ast::PathType) {
    worker.work_on_node(Box::new(&aexpr), &mut |worker| {
        aexpr.path().map_or((), |node|{
            visit_path(worker, node);
        });
    });
}

fn visit_path_segment<'a, D>(worker: &mut worker<D>, aexpr: syntax::ast::PathSegment) {
    worker.work_on_node(Box::new(&aexpr), &mut |worker| {
        worker.work_on_token(aexpr.coloncolon_token());
        match aexpr.name_ref() {
            Some(node) => {
                worker.work_on_node(Box::new(&node), &mut |worker| {});
            }
            None => {}
        }
        aexpr.generic_arg_list().map_or((), |node|{
            ;
        });
        aexpr.param_list().map_or((), |node|{
            visit_param_list(worker, node);
        });
        aexpr.ret_type().map_or((), |node|{
            visit_ret_type(worker, node);
        });
        worker.work_on_token(aexpr.l_angle_token());
        aexpr.path_type().map_or((), |node|{
            visit_path_type(worker, node);
        });
        worker.work_on_token(aexpr.as_token());
        worker.work_on_token(aexpr.r_angle_token());
    });
}

fn visit_path<'a, D>(worker: &mut worker<D>, aexpr: syntax::ast::Path) {
    worker.work_on_node(Box::new(&aexpr), &mut |worker| {
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
    worker.work_on_node(Box::new(&node), &mut |worker| {
        worker.work_on_token(node.l_curly_token());
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
        worker.work_on_token(node.l_curly_token());
    });
}

fn visit_lifetime<'a, D>(worker: &mut worker<D>, node: syntax::ast::Lifetime){
    worker.work_on_node(Box::new(&node), &mut |worker|{
        worker.work_on_token(node.lifetime_ident_token());
    });
}

fn visit_generic_params<'a, D>(worker: &mut worker<D>, node: syntax::ast::GenericParamList){
    worker.work_on_node(Box::new(&node), &mut |worker|{
        worker.work_on_token(node.l_angle_token());

    })
}

fn visit_expr<'a, D>(worker: &mut worker<D>, node: syntax::ast::Expr) {
    match node {
        ArrayExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.l_brack_token());
                for ex in aexpr.exprs() {
                    visit_expr(worker, ex);
                }
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.semicolon_token());
                worker.work_on_token(aexpr.r_brack_token());
            });
        }
        AwaitExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.dot_token());
                worker.work_on_token(aexpr.await_token());
            });
        }
        BinExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.lhs().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.op_token());
                aexpr.rhs().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BoxExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.box_token());
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BreakExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.break_token());
                aexpr.lifetime().map_or((), |lifetime|{
                    visit_lifetime(worker, lifetime);
                });
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        CallExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ClosureExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.for_token());
                aexpr.generic_param_list().map_or((), |node| {
                    visit_generic_params(worker, node);
                });
                worker.work_on_token(aexpr.static_token());
                worker.work_on_token(aexpr.async_token());
                worker.work_on_token(aexpr.move_token());
                aexpr.param_list().map_or((), |node|{
                    visit_param_list(worker, node);
                });
                aexpr.ret_type().map_or((), |node|{
                    visit_ret_type(worker, node);
                });
                aexpr.body().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        CastExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.as_token());
                aexpr.ty().map_or((), |node|{
                    visit_type(worker, node);
                });
            });
        }
        ContinueExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.continue_token());
                aexpr.lifetime().map_or((), |node|{
                    visit_lifetime(worker, node);
                });
            });
        }
        FieldExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.dot_token());
                aexpr.name_ref().map_or((), |node|{
                    worker.work_on_node(Box::new(&node), &mut |worker| {});
                });
            });
        }
        ForExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                worker.work_on_token(aexpr.for_token());
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
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
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
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
            //TODO
        }
        Literal(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
            //TODO
        }
        LoopExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
            //TODO
        }
        MacroExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
            /*TODO*/
        }
        MatchExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
            aexpr.expr().map_or((), |node|{
                visit_expr(worker, node);
            });
        }
        MethodCallExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.receiver().map_or((), |node|{
                    visit_expr(worker, node);
                });
                worker.work_on_token(aexpr.dot_token());
                aexpr.name_ref().map_or((), |node|{
                    worker.work_on_node(Box::new(&node), &mut |worker| {});
                }) 
            });
        }
        ParenExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        PathExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.path().map_or((), |node|{
                    visit_path(worker, node);
                });
            });
        }
        PrefixExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        RangeExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| ());
        }
        RecordExpr(aexpr) => {
            worker.work_on_node(
                Box::new(&aexpr),
                &mut |worker| match aexpr.record_expr_field_list() {
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
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        ReturnExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
       TryExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        TupleExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                for child in aexpr.fields() {
                    visit_expr(worker, child);
                }
            });
        }
        WhileExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
        }
        YieldExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        BlockExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.stmt_list().map_or((), |node| {
                    visit_stmts(worker, node);
                });
            });
        }
        LetExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {
                aexpr.expr().map_or((), |node|{
                    visit_expr(worker, node);
                });
            });
        }
        UnderscoreExpr(aexpr) => {
            worker.work_on_node(Box::new(&aexpr), &mut |worker| {});
        }
    }
}

fn visit_param_list<'a, D>(worker: &mut worker<D>, plist: syntax::ast::ParamList) {

    worker.work_on_node(Box::new(&plist), &mut |worker| {
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

    worker.work_on_node(Box::new(&node), &mut |worker| {
        worker.work_on_token(node.ident_token());
        worker.work_on_token(node.self_token());
    });
}

fn visit_type<'a, D>(worker: &mut worker<D>, node: syntax::ast::Type) {

    worker.work_on_node(Box::new(&node), &mut |worker| {});
    //need to work on the other types TODO

}

fn visit_abi<'a, D>(worker: &mut worker<D>, node: syntax::ast::Abi) {

    worker.work_on_node(Box::new(&node), &mut |worker| {});
    //need to work TODO

}

fn visit_ret_type<'a, D>(worker: &mut worker<D>, node: syntax::ast::RetType) {

    worker.work_on_node(Box::new(&node), &mut |worker| {});
    //need to work TODO

}

fn visit_item<'a, D>(worker: &mut worker<D>, node: syntax::ast::Item) {
    //notcomplete
    match node {
        syntax::ast::Item::Const(node) => {
            worker.work_on_node(Box::new(&node), &mut |worker| {
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
            worker.work_on_node(Box::new(&node), &mut |worker| {
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
        syntax::ast::Item::Impl(node) => worker.work_on_node(Box::new(&node), &mut |worker| {
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

pub fn wraproot<'a, D>(contents: &str, 
        wrap_node_aux: fn(&mut worker<D>, LineIndex, Box<&dyn AstNode>, &mut dyn FnMut(&mut worker<D>)) -> Option<D>,
        wrap_keyword_aux: fn(LineIndex, Option<SyntaxToken>) -> Option<D>)where D: 'a{
    let root = SourceFile::parse(contents).tree();
    let items = root.items();
    let lindex: LineIndex = LineIndex::new(&root.to_string()[..]);

    let mut worker = worker::new(lindex, wrap_node_aux, wrap_keyword_aux);

    for item in items.into_iter() {
        //for now skips Attributes
        //visit(worker, item)
        visit_item(&mut worker, item);
    };
}
