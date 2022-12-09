mod ast0;

use ast0::{info, position_info, token_info, wrap};
use ide_db::defs::OperatorClass;
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax::ast::HasArgList;
use syntax::ast::Path;
use syntax::ast::PathSegment;
use syntax::ast::PathType;
use std::cell::Ref;
use std::collections::HashMap;
use std::ops::Index;
use std::process::Child;
use std::rc::Rc;
use std::vec;
use syntax;
use syntax::ast::AstChildren;
use syntax::ast::Const;
use syntax::ast::Expr;
use syntax::ast::Expr::*;
use syntax::ast::ExprStmt;
use syntax::ast::HasName;
use syntax::ast::ParamList;
use syntax::ast::RecordFieldList;
use syntax::ast::StmtList;
use syntax::ast::TupleFieldList;
use syntax::ast::{AnyHasArgList, AstNode, HasModuleItem, Item, SourceFile, Type};
use syntax::AstToken;
use syntax::SyntaxNode;
use syntax::SyntaxToken;

use self::ast0::bef_aft;
use self::ast0::dummy;

pub use self::ast0::Rnode;
pub use self::ast0::Syntax;
pub use self::ast0::Syntax::{Node, Token};

pub fn wrap_keyword_aux<'a>(lindex: &LineIndex, aexpr: Option<SyntaxToken>) -> Option<Rnode<'a>> {
    //significance of dyn

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
            let wrap: wrap = wrap::new(
                Token(aexpr.clone()),
                info,
                0,
                ast0::mcodekind::MIXED(),
                None,
                bef_aft {},
                AnyHasArgList::can_cast(aexpr.kind()),
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

pub fn wrap_token_aux<'a, K: AstToken>(lindex: &LineIndex, aexpr: Option<K>) -> Option<Rnode<'a>> {
    //significance of dyn

    match aexpr {
        Some(aexpr) => {
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
                Token(aexpr.syntax().clone()),
                info,
                0,
                ast0::mcodekind::MIXED(),
                None,
                bef_aft {},
                AnyHasArgList::can_cast(aexpr.syntax().kind()),
                false,
                false,
                vec![],
            );

            Some(
                Rnode {
                    wrapper: wrap,
                    astnode: Token(aexpr.syntax().clone()),
                    children: vec![],
                }, //cloning is cheap as astnode is
                   //cheap as mentioned in rowan documentation
            )
        }
        None => None,
    }
}

///
/// Next two functions for wrapping nodes
/* fn rewrap_node<'a, K: AstNode>( //This function has been made to rewrap Option<AstNode> to Option<&AstNode>
    lindex: &LineIndex, opt: Option<K>, isSymbolIdent: bool){//This function needs to be reviewed, why is &dyn not required
    match opt{
        Some(t) => { wrap_node_aux(lindex, Some(&t), isSymbolIdent) }
        None => {}
    }
}
*/
pub fn wrap_node_aux<'a, K: AstNode>(
    lindex: &LineIndex,
    aexpr: Option<K>,
    isSymbolIdent: bool,
) -> Option<Rnode<'a>> {
    match aexpr {
        Some(aexpr) => {
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
                isSymbolIdent,
            );
            let wrap: wrap = wrap::new(
                Node(aexpr.syntax().clone()),
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
        None => None,
    }
}

pub fn wrap_node_ref_aux<'a, K: AstNode>(
    lindex: &LineIndex,
    aexpr: Option<&K>,
    isSymbolIdent: bool,
) -> Option<Rnode<'a>> {
    match aexpr {
        Some(aexpr) => {
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
                isSymbolIdent,
            );
            let wrap: wrap = wrap::new(
                Node(aexpr.syntax().clone()),
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
        None => None,
    }
}

fn wrap_path_type<'a>(lindex: &LineIndex, aexpr: Option<PathType>) -> Option<Rnode<'a>>{
    match aexpr{
        Some(aexpr) => {
            let children = vec![wrap_path(lindex,
                (aexpr.path()).as_ref()
                )];
            let mut wrappedp = wrap_node_aux(lindex, Some(aexpr), false).unwrap();
            wrappedp.set_children(children);
            Some(wrappedp)
        }
        None => { None }
    }
}

fn wrap_path_segnment<'a>(lindex: &LineIndex, aexpr: Option<PathSegment>) -> Option<Rnode<'a>>{
    let mut children: Vec<Option<Rnode>> = vec![];
    match aexpr{
        Some(aexpr) => {
            children.push(wrap_keyword_aux(lindex, aexpr.coloncolon_token()));
            children.push(wrap_node_aux(lindex, aexpr.name_ref(), true));
            children.push(wrap_path_type(lindex, aexpr.path_type()));
            let mut wrappedp = wrap_node_aux(lindex, Some(aexpr), false).unwrap();
            wrappedp.set_children(children);
            Some(wrappedp)
        }
        None => { None }
    }
    
}

fn wrap_path<'a>(lindex: &LineIndex, aexpr: Option<&Path>) -> Option<Rnode<'a>> {
    let mut children: Vec<Option<Rnode>> = vec![];
    match aexpr{
        Some(aexpr) => {
            let qualifier = aexpr.qualifier();
            match qualifier{
                Some(q) => {
                    children.push(wrap_path(lindex, Some(&q)));
                }
                None => { children.push(None); }
            }
            let segment = aexpr.segment();
            children.push(wrap_path_segnment(lindex, segment));
            let mut wrappedp = wrap_node_ref_aux(lindex, Some(aexpr), false).unwrap();
            wrappedp.set_children(children);
            Some(wrappedp)
        }
        None => { None }
    }
}

fn wrap_stmts<'a>(lindex: &LineIndex, node: Option<StmtList>) -> Option<Rnode<'a>> {
    let mut children: Vec<Option<Rnode>> = vec![];
    match node {
        Some(node) => {
            for stmt in node.statements() {
                match stmt {
                    syntax::ast::Stmt::ExprStmt(estmt) => {
                        children.push(wrap_expr(lindex, estmt.expr()));
                    }
                    syntax::ast::Stmt::Item(istmt) => children.push(wrap_item(lindex, istmt)),
                    syntax::ast::Stmt::LetStmt(lstmt) => {
                        //TODO
                        //let name = lstmt.to_string();
                        //println!("{name}");
                        children.push(wrap_expr(lindex, lstmt.initializer()));
                        match lstmt.let_else() {
                            Some(le) => children.push(wrap_expr(
                                lindex,
                                match le.block_expr(){
                                    Some(bexpr) => { Some(syntax::ast::Expr::BlockExpr(bexpr)) }
                                    None => { None }
                                }
                            )),
                            None => {}
                        }
                    }
                }
            }
            let mut wrappedstmts = wrap_node_aux(&lindex, Some(node), false).unwrap();
            wrappedstmts.set_children(children);
            Some(wrappedstmts)
        }
        None => None,
    }
}

fn wrap_expr<'a>(lindex: &LineIndex, node: Option<syntax::ast::Expr>) -> Option<Rnode<'a>> {
    let mut children: Vec<Option<Rnode>> = vec![];
    match node {
        Some(node) => {
            match &node {
                ArrayExpr(aexpr) => {
                    for ex in aexpr.exprs() {
                        children.push(wrap_expr(lindex, Some(ex)));
                    }
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                AwaitExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                BinExpr(aexpr) => {}
                BoxExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                BreakExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                CallExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                ClosureExpr(aexpr) => {
                    //todo param
                    children.push(wrap_expr(lindex, aexpr.body()));
                }
                CastExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                ContinueExpr(aexpr) => {}
                FieldExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                ForExpr(aexpr) => {}
                IfExpr(aexpr) => {
                    aexpr.else_branch()
                }
                IndexExpr(aexpr) => {}
                Literal(aexpr) => {}
                LoopExpr(aexpr) => {}
                MacroExpr(aexpr) => { /*TODO*/ }
                MatchExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                MethodCallExpr(aexpr) => {
                    println!("HAOUEODENO");
                    children.push(wrap_expr(&lindex, aexpr.receiver()));
                    children.push(wrap_keyword_aux(lindex, aexpr.dot_token()));
                    children.push(wrap_node_aux(&lindex, aexpr.name_ref(), true));
                }
                ParenExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                PathExpr(aexpr) => {
                    children.push(wrap_path(lindex, 
                        match &aexpr.path(){
                            Some(aexpr) => { Some(aexpr) }
                            None => None
                        }
                    ));
                },
                PrefixExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                RangeExpr(aexpr) => {}
                RecordExpr(aexpr) => match aexpr.record_expr_field_list() {
                    Some(al) => {
                        children.push(wrap_expr(lindex, al.spread()));
                    }
                    None => {}
                },
                RefExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                ReturnExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                TryExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                TupleExpr(aexpr) => {
                    for child in aexpr.fields() {
                        children.push(wrap_expr(lindex, Some(child)))
                    }
                }
                WhileExpr(aexpr) => {}
                YieldExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                BlockExpr(aexpr) => {
                    //let name = aexpr.to_string();
                    //println!("{name}");
                    children.push(wrap_stmts(lindex, aexpr.stmt_list()));
                }
                LetExpr(aexpr) => {
                    children.push(wrap_expr(lindex, aexpr.expr()));
                }
                UnderscoreExpr(aexpr) => {}
            }
            let mut wrappedbody = wrap_node_aux(&lindex, Some(node), false).unwrap();
            wrappedbody.set_children(children);
            Some(wrappedbody)
        }
        None => None,
    }
}

fn wrap_params(lindex: &LineIndex, plist: Option<ParamList>) {
    match plist {
        Some(plist) => {
            wrap_keyword_aux(lindex, plist.l_paren_token());
            wrap_keyword_aux(lindex, plist.comma_token());
            for param in plist.params() {
                //wrap_pat(lindex, param.pat());
                wrap_keyword_aux(lindex, param.colon_token());
                wrap_node_aux(lindex, param.ty(), false);
                wrap_keyword_aux(lindex, param.dotdotdot_token());
            }
            wrap_keyword_aux(lindex, plist.r_paren_token());
            wrap_keyword_aux(lindex, plist.pipe_token());
            wrap_node_aux(lindex, Some(plist), false);
        }
        None => {}
    }
}

fn wrap_item<'a>(lindex: &LineIndex, node: syntax::ast::Item) -> Option<Rnode<'a>> {
    //notcomplete
    let mut children: Vec<Option<Rnode>> = vec![];
    match &node {
        syntax::ast::Item::Const(node) => {
            children.push(wrap_node_aux(&lindex, node.name(), true));
            children.push(wrap_keyword_aux(&lindex, node.default_token()));
            children.push(wrap_keyword_aux(&lindex, node.const_token()));
            children.push(wrap_keyword_aux(&lindex, node.underscore_token()));
            children.push(wrap_keyword_aux(&lindex, node.colon_token()));
            children.push(wrap_node_aux(&lindex, node.ty(), false));//This can have generic arguments
            children.push(wrap_keyword_aux(&lindex, node.eq_token()));
            children.push(wrap_node_aux(&lindex, node.body(), false));
            children.push(wrap_keyword_aux(&lindex, node.semicolon_token()));

            //Adding this at the end so as to avoid
            //moving value until the end
        }
        syntax::ast::Item::Fn(node) => {
            children.push(wrap_expr(
                &lindex,
                match node.body() {
                    Some(body) => { Some(syntax::ast::Expr::BlockExpr(body)) }
                    None => { None }
                }
            ));
        }
        syntax::ast::Item::Impl(node) => {
            println!("Actually enters here");
            match node.assoc_item_list(){
                Some(item) => {
                    for item in item.assoc_items(){
                        match item{
                            syntax::ast::AssocItem::Const(cnt) => {
                                children.push(wrap_item(lindex, syntax::ast::Item::Const(cnt)));
                            }
                            syntax::ast::AssocItem::Fn(f) => {
                                children.push(wrap_item(lindex, syntax::ast::Item::Fn(f)));
                            }
                            syntax::ast::AssocItem::MacroCall(mc) => {
                                children.push(wrap_item(lindex, syntax::ast::Item::MacroCall(mc)));
                            }
                            syntax::ast::AssocItem::TypeAlias(ta) => {
                                children.push(wrap_item(lindex, syntax::ast::Item::TypeAlias(ta)));
                            }
                        }
                        
                    }
                }
                None => {}
            }
            
        }
        _ => {}
    }
    let mut wrappeditem = wrap_node_aux(&lindex, Some(node), false).unwrap();
    wrappeditem.set_children(children);
    Some(wrappeditem)
}

pub fn wraproot(contents: &str) -> Option<Rnode> {
    let root = SourceFile::parse(contents).tree();
    let mut infonode: HashMap<Syntax, wrap> = HashMap::new();
    let mut children: Vec<Option<Rnode>> = vec![];
    let items = root.items();
    let lindex: LineIndex = LineIndex::new(&root.to_string()[..]);
    for item in items {
        //for now skips Attributes
        children.push(wrap_item(&mut &lindex, item.clone()));
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
        Node(root.syntax().clone()),
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
    Some(Rnode::new_root(wrap, Node(root.syntax().clone()), children))
}
