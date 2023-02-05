use itertools::{izip};
use parser::SyntaxKind;

use crate::{parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode};

type Tag = SyntaxKind;


fn tokenf<'a>(node1: &'a mut Snode, node2: &'a mut Rnode) {
    
}

fn workon(node1: &mut Snode, node2: &mut Rnode){
    //only used for nodes with the same number of children
    match node1.wrapper.metavar {
        crate::parsing_cocci::ast0::MetaVar::NoMeta => {}
        _ => {}
    }

    match (node1.kind().is_keyword() || node1.kind().is_punct(),
           node2.kind().is_keyword() || node2.kind().is_punct()) {
        (true, true) => { //this is tokenf
            tokenf(node1, node2);
        },
        (false, false) => {
            assert!(node1.children.len() == node2.children.len());
            let zipped = izip!(node1.children, node2.children);
            for (a, b) in zipped {
                workon(&mut a, &mut b);
            }
        }
        _ => {}
    }
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {
    match (nodeA.unwrap(), nodeB.unwrap()) {
        ((Tag::BIN_EXPR, [lhs1, op1, rhs1]), (Tag::BIN_EXPR, [lhs2, op2, rhs2])) => {}
        _ => {}
    }
}
