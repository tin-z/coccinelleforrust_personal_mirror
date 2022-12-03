mod ast0;

use std::cell::Ref;
use std::collections::HashMap;
use std::rc::Rc;
use ast0::{position_info, token_info, info, wrap};
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax;
use syntax::SyntaxNode;
use syntax::ast::Const;
use syntax::ast::Expr;
use syntax::ast::HasName;
use syntax::ast::RecordFieldList;
use syntax::ast::TupleFieldList;
use syntax::ast::{Item, SourceFile, Type, AnyHasArgList, AstNode, HasModuleItem};
use syntax::ast::Expr::*;

use self::ast0::bef_aft;
use self::ast0::dummy;

fn wrap_node_aux(infonode: &mut HashMap<SyntaxNode, wrap>, lindex: &LineIndex, aexpr: &dyn AstNode){
    
    let sindex: LineCol = lindex.line_col(aexpr.syntax().text_range().start());
    let eindex: LineCol = lindex.line_col(aexpr.syntax().text_range().end());

    let pos_info: position_info = position_info{
        line_start: sindex.line,
        line_end: eindex.line,
        logical_start: 0,//TODO
        logical_end: 0,
        column: sindex.col,
        offset: aexpr.syntax().text_range().start().into()
    };
    let info = info::new(pos_info, 
                                false, false, 
                                vec![], vec![],
                                vec![], vec![],
                                false);
    let wrap: wrap = wrap::new(aexpr.syntax().clone(), info,
                                0, ast0::mcodekind::MIXED(), 
                                Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                false, false, 
                                vec![]);

    infonode.insert(aexpr.syntax().clone(), wrap);
}

fn wrap_expr(infonode: &mut HashMap<SyntaxNode, wrap>, lindex: LineIndex, node: syntax::ast::Expr){
    match node{
        ArrayExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        AwaitExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        BinExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        BlockExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        BoxExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        BreakExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        CallExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        CastExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        ClosureExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        ContinueExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        FieldExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        ForExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        IfExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        IndexExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        Literal(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        LoopExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        MacroExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        MatchExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        MethodCallExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        ParenExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        PathExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        PrefixExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        RangeExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        RecordExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        RefExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        ReturnExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        TryExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        TupleExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        WhileExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        YieldExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        LetExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) },
        UnderscoreExpr(aexpr)=> { wrap_node_aux(infonode, &lindex, &aexpr) }
    }
}

fn wrap_aux(infonode: &mut HashMap<SyntaxNode, wrap>, lindex: LineIndex, node: &syntax::ast::Item) {//notcomplete
    match node {
        syntax::ast::Item::Const(node)=> { 
            wrap_node_aux(infonode, &lindex, node); 
            match node.body(){
                Some(expr) => { wrap_node_aux(infonode, &lindex, &expr); }
                None => { }
            }
        }
        syntax::ast::Item::Enum(node)=> { 
            wrap_node_aux(infonode, &lindex, node);
            wrap_node_aux(infonode, &lindex, &node.name().unwrap());//enum name can never be missing
            let variants = node.variant_list();
            match variants{
                Some(variants) => {
                    for variant in variants.variants(){
                        match variant.field_list(){
                            Some(flist) => { 
                                match flist{
                                    syntax::ast::FieldList::RecordFieldList(rlist)=> {
                                        for rf in rlist.fields(){
                                            wrap_node_aux(infonode, &lindex, &rf.name().unwrap());
                                            wrap_node_aux(infonode, &lindex, &rf.ty().unwrap());
                                        }
                                    }
                                    syntax::ast::FieldList::TupleFieldList(tlist) => {
                                        for tf in tlist.fields(){
                                            wrap_node_aux(infonode, &lindex, &tf.ty().unwrap());
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
        }
        syntax::ast::Item::ExternBlock(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::ExternCrate(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Fn(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Impl(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::MacroCall(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::MacroRules(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::MacroDef(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Module(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Static(node)=> {  wrap_node_aux(infonode, &lindex, node); }
        syntax::ast::Item::Struct(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Trait(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::TypeAlias(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Union(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        syntax::ast::Item::Use(node)=> { wrap_node_aux(infonode, &lindex, node);  }
        }
}


fn wraproot(contents: &str) {
    let root = SourceFile::parse(contents).tree();
    let mut infonode: HashMap<SyntaxNode, wrap> = HashMap::new();
    let items = root.items();
    for item in items {//for now skips Attributes
        let lindex: LineIndex = LineIndex::new(&item.to_string()[..]);
        let sindex: LineCol = lindex.line_col(item.syntax().text_range().start());
        let eindex: LineCol = lindex.line_col(item.syntax().text_range().end());

        let pos_info = position_info {
            line_start: sindex.line,
            line_end: eindex.line,
            logical_start: 0,//TODO
            logical_end: 0,
            column: sindex.col,
            offset: item.syntax().text_range().start().into()
        };

        let info = info::new(pos_info,
                            false, false,
                            vec![], vec![],
                            vec![], vec![],
                            false
                        );

        let wrap: wrap = wrap::new(item.syntax().clone(), info, 0,
                                    ast0::mcodekind::MIXED(), Type::cast(item.syntax().to_owned()),
                                    bef_aft{},//TODO
                                    AnyHasArgList::can_cast(item.syntax().kind()),
                                    false, false, 
                                    vec![]
                            );
        
        infonode.insert(item.syntax().clone(), wrap);
        wrap_aux(&mut infonode, lindex, &item);
    }
}