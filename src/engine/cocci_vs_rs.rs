use itertools::{izip};
use parser::SyntaxKind;
use syntax::TextRange;

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
// We can use Result Object Error ass error codes when it fails
fn loopnodes<'a>(node1: &Snode, node2: &mut Rnode) -> Result<(&'a Snode, &'a Rnode), usize>{
    if node1.kind() != node2.kind() {
        return Err(0)
    }
   
    if node1.children.len() != node2.children.len() {
        return Err(0)
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
                    return Err(0)
            },
            _ => {
                workon(&a, &mut b);
                loopnodes(node1, node2);
            }
        }
    }
    return Ok((node1, node2));
}

//Example function for manual traversal
fn traversenode<'a>(node1: &Snode, node2: &mut Rnode) -> Result<(&'a Snode, &'a Rnode), usize> {
    // Analogous to manually popping out elements like
    // match c1::children1, c2::children2
    if node1.kind() != node2.kind() {
        return Err(0)
    }
   
    if node1.children.len() != node2.children.len() {
        return Err(0)
    }

    //For example we are working on the if node
    match (node1.children, node2.children) {
        ([aifk,aexpr1, aelsek, aexpr2],
        [bifk, bexpr1, belsek, bexpr2]) => {
            
        }
        _ => {}
    }
    tokenf(node1, node2)
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {
    
}

