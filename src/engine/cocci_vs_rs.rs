use itertools::{izip};
use parser::SyntaxKind;

use crate::{parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode, fail, parsing_cocci::ast0::{Fixpos, Mcodekind}, commons::info::ParseInfo};

type Tag = SyntaxKind;

fn checkpos(info: Option<ParseInfo>, mck: Mcodekind, pos: Fixpos) {
    match mck {
        Mcodekind::PLUS(count) => {}
        Mcodekind::MINUS(replacement) => {}
        Mcodekind::CONTEXT(befaft) => {}
        Mcodekind::MIXED(befaft) => {}
    }
}

fn tokenf<'a>(node1: &'a Snode, node2: &'a mut Rnode){//this is 
    //transformation.ml's tokenf
    //info_to_fixpos
    let pos =
    match node2.wrapper.info{
        crate::parsing_rs::ast_rs::ParseInfo::OriginTok(pi) => {
            Fixpos::Real(pi.charpos)
        }
        crate::parsing_rs::ast_rs::ParseInfo::FakeTok(_, (pi, offset)) => {
            Fixpos::Virt(pi.charpos, offset)
        }
    };


}
fn workon(node1: &Snode, node2: &mut Rnode) {
    //check for metavars

}

fn loopnodes(node1: &Snode, node2: &mut Rnode) {
    if node1.kind() != node2.kind() {
        fail!();// just acts as fail for now. Will replace this with
                 // Result<> after sructure is finalized
    }
   
    if node1.children.len() != node2.children.len() {
        fail!();
    }
    let zipped = izip!(node1.children, node2.children);
    for (a, b) in zipped {
        match (a.kind().is_keyword(), b.kind().is_punct(),
        a.kind().is_keyword(), b.kind().is_punct()) {
            (a1, _, a2, _) | // both keywords 
            (_, a1, _, a2) //both puncts
                if a1 && a2 => { 
                    // (the _ because i am not sure 
                    //if a keyword exists that is somehow also a punctuation, 
                    //so subject to change)
                    tokenf(node1, node2);
            },
            (a1, _, a2, _) |  
            (_, a1, _, a2)
                if !a1 && !a2=> { 
                    fail!();
            },
            _ => {
                workon(&a, &mut b);
                loopnodes(node1, node2);
            }
        }
    }
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {
    
}
