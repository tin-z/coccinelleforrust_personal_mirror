use ide_db::{line_index::{LineIndex, LineCol}, LineIndexDatabase};
use syntax::{SyntaxElement, SourceFile, AstNode};

use crate::{parsing_rs::visitor_ast::work_node, commons::info::{PositionInfo, ParseInfo}};

use super::ast_rs;
use super::ast_rs::{Rnode, Wrap};


pub fn fill_wrap<'a>(lindex: &LineIndex, node: &SyntaxElement) -> Wrap<'a> {
    let sindex: LineCol = lindex.line_col(node.text_range().start());

    let parse_info = ParseInfo::new(
        String::new(),
        usize::from(node.text_range().start()),

        sindex.line as usize,
        sindex.col as usize,
        String::new()
    );

    let wrap: Wrap = Wrap::new(
        ast_rs::ParseInfo::OriginTok(parse_info),
        0,
        None,
        super::ast_rs::Danger::NoDanger
    );
    wrap
}

pub fn processrs<'a>(contents: &str) -> Rnode<'a> {
    let lindex = LineIndex::new(contents);
    let root = SourceFile::parse(contents).tree();
    let wrap_node = &|node: SyntaxElement, df: &dyn Fn(&SyntaxElement) -> Vec<Rnode<'a>>| -> Rnode<'a> {
        let wrapped = fill_wrap(&lindex, &node);
        let children = df(&node);
        let rnode = Rnode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children: children,
        };
        rnode
    };
    work_node(wrap_node, SyntaxElement::Node(root.syntax().clone()))
}