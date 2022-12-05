mod ast0;

use ast0::{info, position_info, token_info, wrap};
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use std::cell::Ref;
use std::collections::HashMap;
use std::process::Child;
use std::rc::Rc;
use syntax;
use syntax::ast::AstChildren;
use syntax::ast::Const;
use syntax::ast::Expr;
use syntax::ast::Expr::*;
use syntax::ast::HasName;
use syntax::ast::ParamList;
use syntax::ast::RecordFieldList;
use syntax::ast::TupleFieldList;
use syntax::ast::{AnyHasArgList, AstNode, HasModuleItem, Item, SourceFile, Type};
use syntax::AstToken;
use syntax::SyntaxNode;
use syntax::SyntaxToken;

use self::ast0::bef_aft;
use self::ast0::dummy;
use self::ast0::Rnode;
use self::ast0::Syntax;
pub use self::ast0::Syntax::{Node, Token};

fn wrap_keyword_aux<'a>(
    infonode: &mut HashMap<Syntax, wrap>,
    lindex: &LineIndex,
    aexpr: Option<SyntaxToken>,
) -> Option<Rnode<'a>> {
    //significance of dyn

    match aexpr {
        Some(aexpr) => {
            let sindex: LineCol = lindex.line_col(aexpr.text_range().start());
            let eindex: LineCol = lindex.line_col(aexpr.text_range().end());

            let pos_info: position_info = position_info {
                line_start: sindex.line,
                line_end: eindex.line,
                logical_start: 0, //TODO
                logical_end: 0,
                column: sindex.col,
                offset: aexpr.text_range().start().into(),
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

fn wrap_token_aux<'a, K: AstToken>(
    infonode: &mut HashMap<Syntax, wrap>,
    lindex: &LineIndex,
    aexpr: Option<K>,
) -> Option<Rnode<'a>> {
    //significance of dyn

    match aexpr {
        Some(aexpr) => {
            let sindex: LineCol = lindex.line_col(aexpr.syntax().text_range().start());
            let eindex: LineCol = lindex.line_col(aexpr.syntax().text_range().end());

            let pos_info: position_info = position_info {
                line_start: sindex.line,
                line_end: eindex.line,
                logical_start: 0, //TODO
                logical_end: 0,
                column: sindex.col,
                offset: aexpr.syntax().text_range().start().into(),
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
/* fn rewrap_node<'a, K: AstNode>(infonode: &mut HashMap<Syntax, wrap>, //This function has been made to rewrap Option<AstNode> to Option<&AstNode>
    lindex: &LineIndex, opt: Option<K>, isSymbolIdent: bool){//This function needs to be reviewed, why is &dyn not required
    match opt{
        Some(t) => { wrap_node_aux(infonode, lindex, Some(&t), isSymbolIdent) }
        None => {}
    }
}
*/
fn wrap_node_aux<'a, K: AstNode>(
    infonode: &mut HashMap<Syntax, wrap>,
    lindex: &LineIndex,
    aexpr: Option<K>,
    isSymbolIdent: bool,
) -> Option<Rnode<'a>> {
    match aexpr {
        Some(aexpr) => {
            let sindex: LineCol = lindex.line_col(aexpr.syntax().text_range().start());
            let eindex: LineCol = lindex.line_col(aexpr.syntax().text_range().end());

            let pos_info: position_info = position_info {
                line_start: sindex.line,
                line_end: eindex.line,
                logical_start: 0, //TODO
                logical_end: 0,
                column: sindex.col,
                offset: aexpr.syntax().text_range().start().into(),
            };
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

fn wrap_expr(infonode: &mut HashMap<Syntax, wrap>, lindex: LineIndex, node: syntax::ast::Expr) {
    /* match node{
        ArrayExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        AwaitExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        BinExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        BlockExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        BoxExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        BreakExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        CallExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        CastExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        ClosureExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        ContinueExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        FieldExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        ForExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        IfExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        IndexExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        Literal(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        LoopExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        MacroExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        MatchExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        MethodCallExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        ParenExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        PathExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        PrefixExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        RangeExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        RecordExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        RefExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        ReturnExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        TryExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        TupleExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        WhileExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        YieldExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        LetExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) },
        UnderscoreExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, Some(aexpr), false) }
    } */
}

fn wrap_params(infonode: &mut HashMap<Syntax, wrap>, lindex: &LineIndex, plist: Option<ParamList>) {
    match plist {
        Some(plist) => {
            wrap_keyword_aux(infonode, lindex, plist.l_paren_token());
            wrap_keyword_aux(infonode, lindex, plist.comma_token());
            for param in plist.params() {
                //wrap_pat(infonode, lindex, param.pat());
                wrap_keyword_aux(infonode, lindex, param.colon_token());
                wrap_node_aux(infonode, lindex, param.ty(), false);
                wrap_keyword_aux(infonode, lindex, param.dotdotdot_token());
            }
            wrap_keyword_aux(infonode, lindex, plist.r_paren_token());
            wrap_keyword_aux(infonode, lindex, plist.pipe_token());
            wrap_node_aux(infonode, lindex, Some(plist), false);
        }
        None => {}
    }
}

fn wrap_item<'a>(
    infonode: &mut HashMap<Syntax, wrap>,
    lindex: &LineIndex,
    node: syntax::ast::Item,
) -> Option<Rnode<'a>> {
    //notcomplete
    match node {
        syntax::ast::Item::Const(node) => {
            let mut children: Vec<Option<Rnode>> = vec![];
            /// If a child is not present None is pushed
            children.push(wrap_node_aux(infonode, &lindex, node.name(), true));
            children.push(wrap_keyword_aux(infonode, &lindex, node.default_token()));
            children.push(wrap_keyword_aux(infonode, &lindex, node.const_token()));
            children.push(wrap_keyword_aux(infonode, &lindex, node.underscore_token()));
            children.push(wrap_keyword_aux(infonode, &lindex, node.colon_token()));
            children.push(wrap_node_aux(infonode, &lindex, node.name(), true));
            children.push(wrap_node_aux(infonode, &lindex, node.ty(), false));
            children.push(wrap_keyword_aux(infonode, &lindex, node.eq_token()));
            children.push(wrap_node_aux(infonode, &lindex, node.body(), false));
            children.push(wrap_keyword_aux(infonode, &lindex, node.semicolon_token()));

            //Adding this at the end so as to avoid
            //moving value until the end
            let mut wrappeditem = wrap_node_aux(infonode, &lindex, Some(node), false).unwrap();
            wrappeditem.set_children(children);
            Some(wrappeditem)
        }
        syntax::ast::Item::Fn(node) => {
            let mut wrappeditem = wrap_node_aux(infonode, &lindex, Some(node), false).unwrap();
            Some(wrappeditem)
        }
        _ => None, /*syntax::ast::Item::Enum(node)=> {
                       wrap_node_aux(infonode, &lindex, node.name(), true);//enum name can never be missing
                       let variants = node.variant_list();
                       match variants{
                           Some(variants) => {
                               for variant in variants.variants(){
                                   match variant.field_list(){
                                       Some(flist) => {
                                           match flist{
                                               syntax::ast::FieldList::RecordFieldList(rlist)=> {
                                                   for rf in rlist.fields(){
                                                       wrap_node_aux(infonode, &lindex, rf.name(), true);
                                                       wrap_node_aux(infonode, &lindex, rf.ty(), false);
                                                   }
                                               }
                                               syntax::ast::FieldList::TupleFieldList(tlist) => {
                                                   for tf in tlist.fields(){
                                                       wrap_node_aux(infonode, &lindex, tf.ty(), false);
                                                   }
                                               }
                                           }
                                       }
                                       None => {}
                                   }
                               }
                           }
                           None => {}
                       }
                       wrap_node_aux(infonode, &lindex, Some(node), false);
                   }
                   syntax::ast::Item::ExternBlock(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::ExternCrate(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Fn(node)=> {
                       wrap_keyword_aux(infonode, &lindex, node.default_token());
                       wrap_keyword_aux(infonode, &lindex, node.const_token());
                       wrap_keyword_aux(infonode, &lindex, node.async_token());
                       wrap_keyword_aux(infonode, &lindex, node.unsafe_token());
                       wrap_keyword_aux(infonode, &lindex, node.fn_token());
                       wrap_node_aux(infonode, &lindex, node.name(), true);
                       wrap_node_aux(infonode, &lindex, Some(node), false);

                   }
                   syntax::ast::Item::Impl(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::MacroCall(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::MacroRules(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::MacroDef(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Module(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Static(node)=> {  wrap_node_aux(infonode, &lindex, Some(node), false); }
                   syntax::ast::Item::Struct(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Trait(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::TypeAlias(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Union(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   syntax::ast::Item::Use(node)=> { wrap_node_aux(infonode, &lindex, Some(node), false);  }
                   }*/
    }
}

pub fn wraproot(contents: &str) -> Option<Rnode> {
    let root = SourceFile::parse(contents).tree();
    let mut infonode: HashMap<Syntax, wrap> = HashMap::new();
    let mut children: Vec<Option<Rnode>> = vec![];
    let items = root.items();
    let lindex: LineIndex = LineIndex::new(&root.to_string()[..]);
    for item in items {
        //for now skips Attributes
        children.push(wrap_item(&mut infonode, &lindex, item.clone()));
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
