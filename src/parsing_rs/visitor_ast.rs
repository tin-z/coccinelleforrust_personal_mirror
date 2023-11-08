// SPDX-License-Identifier: GPL-2.0

use itertools::Itertools;
use ra_parser::SyntaxKind;
use ra_syntax::SyntaxElement;
use std::vec;

use crate::commons::util::{attach_spaces_back, attach_spaces_front, workrnode};

use super::{ast_rs::Rnode, parse_rs::processrs};
type Tag = SyntaxKind;

fn ttree_to_expr_list(tt: String) -> Option<Vec<Rnode>> {
    let wrapped = format!(
        "fn func() {{
            fcall({})
        }}",
        tt
    );

    let mut rnode = match processrs(&wrapped) {
        Ok(node) => node,
        Err(_) => {
            //In this case the macro is not function like
            //and the tokentree cannot be parsed like function
            //arguments

            return None;
        }
    };

    let mut args = rnode.children[0] //fn
        .children[3] //blockexpr
        .children[0] //stmtlist
        .children[1] //callexpr
        .children
        .remove(1); //arglist

    //removing sorrounding brackets of fcall
    args.children.remove(0);
    args.children.remove(args.children.len() - 1);

    let info = args.children[0].wrapper.info.clone();

    //This makes it as if the expression starts at the start
    //of the file. Later an offset will be added in the calling
    //function
    args.children.iter_mut().for_each(|x| {
        workrnode(x, &mut |node| {
            node.wrapper.info.subtract(info.clone());
            true
        });
    });

    return Some(args.children);

    //let exprlist = node.chil;
}

pub fn work_node<'a>(
    wrap_node: &dyn Fn(SyntaxElement, &dyn Fn(&SyntaxElement) -> Vec<Rnode>) -> Rnode,
    node: SyntaxElement,
) -> Rnode {
    wrap_node(node, &|node| -> Vec<Rnode> {
        let mut children: Vec<Rnode> = vec![];
        let mut estrings: String = String::new();

        match node {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    match child.kind() {
                        Tag::WHITESPACE | Tag::COMMENT => {
                            estrings.push_str(child.to_string().as_str());
                        }
                        Tag::TOKEN_TREE => {
                            //Macros being artificially stitched in
                            let mut exprlist =
                                match ttree_to_expr_list(child.as_node().unwrap().to_string()) {
                                    Some(exprlist) => exprlist,
                                    None => {
                                        //Macros could not be parsed
                                        let rnode = work_node(wrap_node, child);
                                        children.push(rnode);
                                        continue;
                                    }
                                };
                            
                            let info = work_node(wrap_node, child).wrapper.info.clone(); //Currently clones for macros

                            //Adding the offset to the expressions
                            exprlist.iter_mut().for_each(|x| {
                                workrnode(x, &mut |node| {
                                    node.wrapper.info.add(info.clone());
                                    true
                                })
                            });
                            children.extend(exprlist);
                        }
                        _ => {
                            let mut newnode = work_node(wrap_node, child);
                            if children.len() != 0 {
                                if estrings.contains("/*COCCIVAR*/") {
                                    //Only in case of this special variable which has been
                                    //injected at rnode.unformatted() should it be attached to nodes
                                    //that come after it

                                    attach_spaces_front(&mut newnode, String::from("/*COCCIVAR*/"));

                                    //Takes only spaces coming before COCCIVAR
                                    //Anything after COCCIVAR in that line is unformatted
                                    estrings =
                                        estrings.split("/*COCCIVAR*/").collect_vec()[0].to_string();
                                }
                                attach_spaces_back(children.last_mut().unwrap(), estrings);
                            } else {
                                attach_spaces_front(&mut newnode, estrings);
                            }
                            children.push(newnode);
                            estrings = String::new();
                        }
                    }
                }
                if estrings.len() != 0 {
                    attach_spaces_front(children.last_mut().unwrap(), estrings);
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        children
    })
}
