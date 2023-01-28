use itertools::{izip};
use parser::SyntaxKind;

use crate::parsing_rs::ast_rs::Rnode;

type Tag = SyntaxKind;

fn tokenf(node1: &Rnode, node2: &Rnode) -> bool {
    node1.astnode.to_string() == node2.astnode.to_string()//we probably need to change string matching?
}

fn match_rnode_strict(node1: &Rnode, node2: &Rnode) -> bool {
    //this code is FAR from workable, I just wanted to prototype how it would look like
    //but it compiles
    let truity: bool = true;
    for (child1, child2) in izip!(&node1.children, &node2.children) {
        truity &= 
            match (child1.unwrap(), child2.unwrap()) {
                ((Tag::PATH_EXPR, [a]), (Tag::PATH_EXPR, [b])) => {//for vairblaes
                    // A Path expression is a fully/partially qualified variable/item 
                    // https://doc.rust-lang.org/reference/expressions/path-expr.html
                    // So we use simple string matching

                    a.astnode.to_string() == b.astnode.to_string()
                },
                ((Tag::LET_EXPR, [lta, pata, eqa, expa]),//for two let statements
                 (Tag::LET_EXPR, [ltb, patb, eqb, expb])) => {
                    tokenf(lta, ltb) &&
                    match_rnode_strict(pata, patb) &&
                    tokenf(eqa, eqb) &&
                    match_rnode_strict(expa, expb)
                }
                _ => { return true }
            };
    }
    truity
}

pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {
    match (nodeA.unwrap(), nodeB.unwrap()) {
        ((Tag::BIN_EXPR, [lhs1, op1, rhs1]), 
         (Tag::BIN_EXPR, [lhs2, op2, rhs2])) => {
            
        }
        _ => {}
    }
}