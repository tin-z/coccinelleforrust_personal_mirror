// SPDX-License-Identifier: GPL-2.0

use std::vec;
use parser::SyntaxKind;
use syntax;
use syntax::SyntaxElement;

use super::ast_rs::Rnode;

type Tag = SyntaxKind;

pub fn work_node<'a>(
    do_stuff: &dyn Fn(SyntaxElement, String, &dyn Fn(&SyntaxElement) -> Vec<Rnode>) -> Rnode,
    estrings: String,
    node: SyntaxElement,
) -> Rnode {
    do_stuff(node, estrings.clone(), &|node| -> Vec<Rnode> {
        let mut children = vec![];
        let mut estrings: String = String::new();
        //let mut children = vec![];
        match node {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    match child.kind() {
                        Tag::WHITESPACE | Tag::COMMENT=> {
                            estrings.push_str(child.to_string().as_str());
                        }
                        _ => {
                            children.push(work_node(do_stuff, estrings, child));
                            estrings = String::new();
                        }
                    }
                    //children.push(node);
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        if !estrings.is_empty() {
            //if estrings is not empty then there have been comments
            //and comments cannot exists in a level by themselves
            //so unwrap is justified
            if children.len()!=0 {
                children.last_mut().unwrap().wrapper.wspaces.1 = estrings;
            }
        }
        children
    })
}