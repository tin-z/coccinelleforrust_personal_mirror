mod ast0;


use std::cell::Ref;
use std::collections::HashMap;
use ast0::{position_info, token_info, info, wrap};
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax;
use syntax::SyntaxNode;
use syntax::ast::Expr;
use syntax::ast::{Item, SourceFile, Type, AnyHasArgList, AstNode, HasModuleItem};
use syntax::ast::Expr::*;

use self::ast0::bef_aft;
use self::ast0::dummy;

fn wrap_expr(lindex: LineIndex, node: syntax::ast::Expr){
    match node{
        ArrayExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        AwaitExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        BinExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        BlockExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        BoxExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        BreakExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        CallExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        CastExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        ClosureExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        ContinueExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        FieldExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        ForExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        IfExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        IndexExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        Literal(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        LoopExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        MacroExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        MatchExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        MethodCallExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        ParenExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        PathExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        PrefixExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        RangeExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        RecordExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        RefExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        ReturnExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        TryExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        TupleExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        WhileExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        YieldExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        LetExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
        UnderscoreExpr(aexpr)=> {

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
            let wrap: wrap = wrap::new(&aexpr.syntax(), info,
                                        0, ast0::mcodekind::MIXED(), 
                                        Type::cast(aexpr.syntax().to_owned()), bef_aft{}, 
                                        AnyHasArgList::can_cast(aexpr.syntax().kind()), 
                                        false, false, 
                                        vec![]);
        },
    }
}

fn wrap_aux(lindex: LineIndex, node: syntax::ast::Item) {//notcomplete
    // for item in node.children() {
    //     let sindex: LineCol = lindex.line_col(item.text_range().start());
    //     let eindex: LineCol = lindex.line_col(item.text_range().end());
    //     let pos_info = position_info {
    //         line_start: sindex.line,
    //         line_end: eindex.line,
    //         logical_start: 0,
    //         logical_end: 0,
    //         column: sindex.col,
    //         offset: item.text_range().start().into(), //what is the offset?
    //     };
    // }
}


fn wraproot(contents: &str) {
    let root = SourceFile::parse(contents).tree();
    let mut upto: &str;
    let mut lino = 1; //linenumber
    let mut cono = 1; //column number

    for item in root.items() {//for now skips Attributes
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
        
        let wrap: wrap = wrap::new(&item.syntax(), info, 0,
                                    ast0::mcodekind::MIXED(), Type::cast(item.syntax().to_owned()),
                                    bef_aft{},//TODO
                                    AnyHasArgList::can_cast(item.syntax().kind()),
                                    false, false, 
                                    vec![]
                            );

        wrap_aux(lindex, item);
    }
}