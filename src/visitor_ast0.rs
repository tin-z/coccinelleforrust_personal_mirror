mod ast0;

use std::ptr::null;

use ast0::{position_info, token_info, info, wrap};
use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax;
use syntax::SyntaxNode;
use syntax::ast::*;
use syntax::SourceFile;

use self::ast0::bef_aft;
use self::ast0::dummy;

fn parse_aux(lindex: LineIndex, node: AstNode) {//notcomplete
    for item in node.children() {
        let sindex: LineCol = lindex.line_col(item.text_range().start());
        let eindex: LineCol = lindex.line_col(item.text_range().end());
        let pos_info = position_info {
            line_start: sindex.line,
            line_end: eindex.line,
            logical_start: 0,
            logical_end: 0,
            column: sindex.col,
            offset: item.text_range().start().into(), //what is the offset?
        };
    }
}

fn parse(contents: &str) {
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

        let info = info {
            pos_info: pos_info,
            attachable_start: false, attachable_end: false,
            mcode_start: vec![], mcode_end: vec![],
            strings_before: vec![], strings_after: vec![],
            isSymbolIdent: false
        };

        let wrap = wrap { 
            node: &item.syntax(),
            info: info,
            index: 0,
            mcodekind: ast0::mcodekind::MIXED(),
            exp_ty: Type::cast(item.syntax().to_owned()),
            bef_aft: bef_aft{},//TODO
            true_if_arg: AnyHasArgList::can_cast(item.syntax().kind()),
            true_if_test: false,//inquire
            true_if_test_exp: false,//inquire
            iso_info: vec![],
        };

        parse_aux(lindex, item);//function not complete
    }
}
