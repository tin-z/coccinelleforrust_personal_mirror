// SPDX-License-Identifier: GPL-2.0

use ra_ide_db::line_index::LineIndex;
use ra_parser::SyntaxKind;
use std::vec;
use ra_syntax::SyntaxElement;

use super::ast0::Snode;

type Tag = SyntaxKind;

pub fn work_node<'a>(
    lindex: &LineIndex,
    wrap_node: &dyn Fn(
        &LineIndex,
        SyntaxElement,
        Option<String>,
        &dyn Fn(&SyntaxElement) -> Vec<Snode>,
    ) -> Snode,
    node: SyntaxElement,
    modkind: Option<String>,
) -> Snode {
    wrap_node(lindex, node, modkind, &|node| -> Vec<Snode> {
        let mut children = vec![];
        //let mut children = vec![];
        match node {
            SyntaxElement::Node(node) => {
                let mut modkind: Option<String> = None;
                for child in node.children_with_tokens() {
                    match child.kind() {
                        Tag::WHITESPACE => {}
                        Tag::COMMENT => {
                            let commlen: usize = child.text_range().len().into();
                            if commlen == 5 && lindex.line_col(child.text_range().start()).col == 0
                            {
                                //checks for /*?*/ modifiers
                                modkind =
                                    Some(String::from(child.to_string().as_bytes()[2] as char));
                                //in the next iteration the node gets the modkind
                            }
                        }
                        _ => {
                            children.push(work_node(lindex, wrap_node, child, modkind));
                            modkind = None;
                        }
                    }
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        children
    })
}
