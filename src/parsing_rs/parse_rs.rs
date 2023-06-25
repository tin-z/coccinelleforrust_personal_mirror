use ide_db::{
    line_index::{LineCol, LineIndex},
    LineIndexDatabase,
};
use parser::SyntaxKind;
use syntax::{AstNode, SourceFile, SyntaxElement};

use crate::{
    commons::info::{ParseInfo, PositionInfo},
    parsing_rs::visitor_ast::work_node,
};

use super::ast_rs;
use super::ast_rs::{Rnode, Wrap};

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> Wrap {
    let sindex: LineCol = lindex.line_col(node.text_range().start());

    let parse_info = ParseInfo::new(
        String::new(),
        usize::from(node.text_range().start()),
        usize::from(node.text_range().end()),
        sindex.line as usize,
        sindex.col as usize,
        String::new(),
    );

    let wrap: Wrap = Wrap::new(parse_info, 0, None, super::ast_rs::Danger::NoDanger);
    wrap
}

pub fn processrs(contents: &str) -> Rnode {
    //TODO put this in ast_rs.rs
    let lindex = LineIndex::new(contents);
    let root = SourceFile::parse(contents).tree();
    let wrap_node = &|node: SyntaxElement,
                      estring: String,
                      df: &dyn Fn(&SyntaxElement) -> Vec<Rnode>|
     -> Rnode {
        let mut wrapped = fill_wrap(&lindex, &node);
        wrapped.wspaces.0 = estring;
        let children = df(&node);
        let rnode = Rnode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children: children,
        };
        if rnode.kind() == SyntaxKind::EXPR_STMT && rnode.children.len() == 1 {
            // this means there is an expression statement without a ; at the end
            //the reason these are removed because rust-analyzer seems to alter between
            //assigning ExprStmt and IfExprs(maybe others too)
            let mut expr = rnode.children.into_iter().next().unwrap();
            expr.wrapper.wspaces = rnode.wrapper.wspaces;
            return expr;
        }
        rnode
    };
    work_node(
        wrap_node,
        String::new(),
        SyntaxElement::Node(root.syntax().clone()),
    )
}