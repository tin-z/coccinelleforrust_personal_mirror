use itertools::{izip};
use parser::SyntaxKind;
use syntax::TextRange;

use crate::{parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode, fail, parsing_cocci::ast0::{Fixpos, Mcodekind}, commons::info::ParseInfo};

type Tag = SyntaxKind;
type CheckResult<'a> = Result<(&'a Snode, &'a mut Rnode), usize>;

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
fn workon<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a>{
    // Metavar checking will be done inside the match
    // block below

    let kind = node1.kind();
    assert!(kind==node2.kind());
    match kind {// Each kind of node will  be
                // treated specifically and special
                // treatment will be "handed" out as
                // necessary
        Tag::IF_EXPR => {
            let [aifk,aexpr1, aelsek, aexpr2] = &mut node1.children[..];
            let [bifk, bexpr1, belsek, bexpr2] = &mut node2.children[..];
            // All the tokens will be treated seperately by loopnodes
            
        }
        _ => {
            
        }
    }

}
// We can use Result Object Error ass error codes when it fails
fn loopnodes<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a>{
    if node1.children.len() != node2.children.len() {
        return Err(0)
    }
    let zipped = izip!(node1.children, node2.children);
    for (a, b) in zipped {
        let akind = a.kind();
        let bkind = b.kind();
        let aisk = akind.is_keyword();
        let aisp = akind.is_punct();
        let bisk = bkind.is_keyword();
        let bisp = bkind.is_punct();

        if akind != bkind {
            return Err(0)
        }
        else if aisk || aisp || bisk || bisp {// if anyone is a keyword, then it
                                              // either it must be treated with tokenf
                                              // or fail
            if aisk && bisk || aisp && bisp {
                // (the _ because i am not sure 
                //if a keyword exists that is somehow also a punctuation, 
                //so subject to change)
                tokenf(node1, node2); 
            }
            else {
                return Err(0);
            }
        }
        else {
            if let Err(a) = 
            workon(&a, &mut b) {
                return Err(a);
            }; 
            loopnodes(&mut a, &mut b);
        }
    }
    return Ok((node1, node2));
}

//Example function for manual traversal
fn traversenode<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a> {
    // Analogous to manually popping out elements like
    // match c1::children1, c2::children2
    if node1.kind() != node2.kind() ||
       node1.children.len() != node2.children.len() {
        return Err(0)
    }

    //For example we are working on the if node
    match (&mut node1.children[..], &mut node2.children[..]) {
        ([aifk,aexpr1, aelsek, aexpr2],
         [bifk, bexpr1, belsek, bexpr2]) => {
            tokenf(aifk, bifk);
            //...
            Ok((node1, node2));// NOT COMPLETED
        }
        _ => {}
    }
    Err(1);
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {
    
}

