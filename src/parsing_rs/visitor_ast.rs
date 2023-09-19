// SPDX-License-Identifier: GPL-2.0

use ra_parser::SyntaxKind;
use ra_syntax::SyntaxElement;
use std::vec;

use super::ast_rs::Rnode;

type Tag = SyntaxKind;

pub fn work_node<'a>(
    wrap_node: &dyn Fn(SyntaxElement, &dyn Fn(&SyntaxElement) -> Vec<Rnode>) -> Rnode,
    node: SyntaxElement,
) -> Rnode {
    wrap_node(node, &|node| -> Vec<Rnode> {
        let mut children: Vec<Rnode> = vec![];
        let mut estrings: String = String::new();
        //let mut children = vec![];
        match node {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    match child.kind() {
                        Tag::WHITESPACE | Tag::COMMENT => {
                            estrings.push_str(child.to_string().as_str());
                        }
                        _ => {
                            let mut newnode = work_node(wrap_node, child);
                            if children.len() != 0 {
                                if estrings.contains("/*COCCIVAR*/") {
                                    //Only in case of this special variable which has been
                                    //injected at rnode.unformatted() should it be attached to nodes
                                    //that come after it

                                    newnode.wrapper.wspaces.0 = String::from("/*COCCIVAR*/");
                                    estrings = estrings.replace("/*COCCIVAR*/", "");
                                }
                                children.last_mut().unwrap().wrapper.wspaces.1 = estrings;
                            } else {
                                newnode.wrapper.wspaces.0 = estrings;
                            }
                            children.push(newnode);
                            estrings = String::new();
                        }
                    }
                }
                if estrings.len()!=0 {
                    children.last_mut().unwrap().wrapper.wspaces.1 = estrings;
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        children
    })
}
