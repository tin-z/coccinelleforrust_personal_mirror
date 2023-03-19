use itertools::izip;
use parser::SyntaxKind;
use syntax::{TextRange, ast::Meta};

use crate::{
    commons::info::ParseInfo,
    fail,
    parsing_cocci::ast0::Snode,
    parsing_cocci::ast0::{Fixpos, Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

type Tag = SyntaxKind;
type CheckResult<'a> = Result<(&'a Snode, &'a mut Rnode), usize>;
type MetavarBinding = ((String, String), Tag);

struct Tin {
    binding: MetavarBinding,
    binding0: MetavarBinding
}

type Tout<'a> = Vec<((&'a Snode, &'a mut Rnode), MetavarBinding)>;




fn checkpos(info: Option<ParseInfo>, mck: Mcodekind, pos: Fixpos) {
    match mck {
        Mcodekind::PLUS(count) => {}
        Mcodekind::MINUS(replacement) => {}
        Mcodekind::CONTEXT(befaft) => {}
        Mcodekind::MIXED(befaft) => {}
    }
}

fn pos_variables<'a, Tin, Tout>(
    tin: Tin,
    node1: &'a Snode,
    node2: Option<&'a mut Rnode>,
    /*finish's type*/
) -> Tout {
}

fn is_fake(node1: &mut Rnode) -> bool {
    false
}

fn tokenf<'a>(node1: &'a Snode, node2: &'a mut Rnode) -> impl FnMut(Tin) -> Tout<'a> {
    // this is
    // Tout will have the generic types in itself
    // ie ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    fn retfunc<'a>(tin: Tin) -> Tout<'a> {
        /*

        This will be used later(hopefully)
        let pos = match node2.wrapper.info {
            crate::parsing_rs::ast_rs::ParseInfo::OriginTok(pi) => Fixpos::Real(pi.charpos),
            crate::parsing_rs::ast_rs::ParseInfo::FakeTok(_, (pi, offset)) => {
                Fixpos::Virt(pi.charpos, offset)
            }
        };
        //Fixpos code
        let finish = |tin: Tin| {};
        pos_variables(
            tin,
            node1,
            if is_fake(node2) { None } else { Some(node2) },
            finish,
        )
        */
        vec![((node1, node2), tin.binding)]
    };
    retfunc
}

fn workon<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a> {
    // Metavar checking will be done inside the match
    // block below

    let kind = node1.kind();
    assert!(kind == node2.kind());
    match kind {
        // Each kind of node will  be
        // treated specifically and special
        // treatment will be "handed" out as
        // necessary
        Tag::IF_EXPR => {
            let [aifk, aexpr1, aelsek, aexpr2] = &mut node1.children[..];
            let [bifk, bexpr1, belsek, bexpr2] = &mut node2.children[..];
            // All the tokens will be treated seperately by loopnodes
        }
        _ => {}
    }
}
// We can use Result Object Error ass error codes when it fails
fn loopnodes<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a> {
    if node1.children.len() != node2.children.len() {
        return Err(0);
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
            return Err(0);
        } else if aisk || aisp || bisk || bisp {
            // if anyone is a keyword, then it
            // either it must be treated with tokenf
            // or fail
            if aisk && bisk || aisp && bisp {
                // (the _ because i am not sure
                //if a keyword exists that is somehow also a punctuation,
                //so subject to change)
                tokenf(node1, node2);
            } else {
                return Err(0);
            }
        } else {
            if let Err(a) = workon(&a, &mut b) {
                return Err(a);
            }; //if error will propagate
            loopnodes(&mut a, &mut b);
        }
    }
    return Ok((node1, node2));
}

//Example function for manual traversal
fn traversenode<'a>(node1: &Snode, node2: &mut Rnode) -> CheckResult<'a> {
    // Analogous to manually popping out elements like
    // match c1::children1, c2::children2
    if node1.kind() != node2.kind() || node1.children.len() != node2.children.len() {
        return Err(0);
    }

    //For example we are working on the if node
    match (&mut node1.children[..], &mut node2.children[..]) {
        ([aifk, aexpr1, aelsek, aexpr2], [bifk, bexpr1, belsek, bexpr2]) => {
            tokenf(aifk, bifk);
            //...
            return Ok((node1, node2)); // NOT COMPLETED
        }
        _ => {}
    }
    Err(1)
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {}
