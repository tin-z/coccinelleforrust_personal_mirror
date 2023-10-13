// SPDX-License-Identifier: GPL-2.0

use ra_ide_db::line_index::{LineCol, LineIndex};
use ra_syntax::{SourceFile, SyntaxElement, SyntaxNode};

use crate::{commons::info::ParseInfo, parsing_rs::visitor_ast::work_node};

use super::ast_rs::{Rnode, Wrap};

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> Wrap {
    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());

    let parse_info = ParseInfo::new(
        String::new(),
        usize::from(node.text_range().start()),
        usize::from(node.text_range().end()),
        sindex.line as usize,
        eindex.line as usize,
        sindex.col as usize,
        String::new(),
    );

    let wrap: Wrap = Wrap::new(parse_info, 0, super::ast_rs::Danger::NoDanger);
    wrap
}

pub fn processrswithsemantics(contents: &str, rnode: SyntaxNode) -> Result<Rnode, String> {
    //TODO put this in ast_rs.rs
    let lindex = LineIndex::new(contents);

    let wrap_node = &|node: SyntaxElement,
                      df: &dyn Fn(&SyntaxElement) -> Vec<Rnode>|
     -> Rnode {
        let wrapped = fill_wrap(&lindex, &node);
        let kind = node.kind();
        let children = df(&node);
        let rnode = Rnode::new(wrapped, Some(node), kind, children);
        rnode
    };
    Ok(work_node(wrap_node, SyntaxElement::Node(rnode)))
}

pub fn processrs(contents: &str) -> Result<Rnode, String> {
    //TODO put this in ast_rs.rs
    let lindex = LineIndex::new(contents);
    let parse = SourceFile::parse(contents);
    let errors = parse.errors();

    if errors.len() != 0 {
        let mut errorstr = String::new();
        errorstr.push_str(contents);
        errorstr.push('\n');
        for error in errors {
            let lindex = lindex.line_col(error.range().start());
            errorstr.push_str(&format!(
                "Error : {} at line: {}, col {}\n",
                error.to_string(),
                lindex.line,
                lindex.col
            ));
        }
        return Err(errorstr);
    }
    let root = parse.syntax_node();

    let wrap_node = &|node: SyntaxElement,
                      df: &dyn Fn(&SyntaxElement) -> Vec<Rnode>|
     -> Rnode {
        let wrapped = fill_wrap(&lindex, &node);
        let children = df(&node);
        let kind = node.kind();
        let node = if children.len() == 0 { Some(node) } else { None };
        let rnode = Rnode::new(
            wrapped, node, //Change this to SyntaxElement
            kind, children,
        );
        rnode
    };

    Ok(work_node(wrap_node,  SyntaxElement::Node(root)))
}
