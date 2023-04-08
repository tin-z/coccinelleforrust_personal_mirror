use std::vec;

use itertools::{izip, Itertools};
use parser::SyntaxKind;
use regex::Match;
use syntax::{ast::Meta, TextRange};

use crate::{
    commons::info::ParseInfo,
    fail,
    parsing_cocci::ast0::Snode,
    parsing_cocci::ast0::{Fixpos, Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

type Tag = SyntaxKind;
type MatchedNode<'a> = (&'a Snode, &'a mut Rnode);
type CheckResult<'a> = Result<MatchedNode<'a>, usize>;
type MetavarBinding<'a> = (&'a Snode, &'a Rnode);

struct Tin<'a> {
    binding: Vec<MetavarBinding<'a>>,
    binding0: Vec<MetavarBinding<'a>>,
}

//Name is subject to change obv
struct MatchedNodes<'a>(Vec<(&'a Snode, &'a mut Rnode)>);

enum MetavarMatch{
    Fail,
    Maybe,
    Match
}

impl<'a> IntoIterator for MatchedNodes<'a> {
    type Item = (&'a Snode, &'a mut Rnode);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> MatchedNodes<'a> {
    fn bind<F: Fn(MatchedNode, Tin) -> Tout<'a>>(self, f: F, tin: Tin) -> Tout<'a> {
        let mut ret = vec![];
        for i in self {
            ret.push(f(i, tin));
        }
        ret.into_iter().flatten().collect_vec()
    }

    fn empty() -> MatchedNodes<'a> {
        MatchedNodes(vec![])
    }
}

type Tout<'a> = Vec<(MatchedNode<'a>, &'a Vec<MetavarBinding<'a>>)>;

fn checkpos(info: Option<ParseInfo>, mck: Mcodekind, pos: Fixpos) {
    match mck {
        Mcodekind::PLUS(count) => {}
        Mcodekind::MINUS(replacement) => {}
        Mcodekind::CONTEXT(befaft) => {}
        Mcodekind::MIXED(befaft) => {}
    }
}

fn is_fake(node1: &mut Rnode) -> bool {
    false
}

fn tokenf<'a>(node1: &'a Snode, node2: &'a mut Rnode, tin: &'a Tin) -> Tout<'a> {
    // this is
    // Tout will have the generic types in itself
    // ie ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![((node1, node2), &tin.binding)]
}

struct Looper<'a> {
    tokenf: fn(&'a Snode, &'a mut Rnode, tin: &'a mut Tin) -> Tout<'a>,
    workontmp: fn(&'a Snode, &'a mut Rnode) -> usize,//this is basically what >>=/bind does
}

impl<'a> Looper<'a> {
    fn loopnodes(&self, node1: &'a Snode, node2: &'a mut Rnode) -> Tin {
        // It has to be checked before if these two node tags match
        if node1.kind()!=node2.kind() || 
           node1.children.len() != node2.children.len() {
            fail!();
        }
    
        let zipped = izip!(&node1.children, &mut node2.children);
        let mut prev: Tout;
        let mut tin = &mut Tin { binding: vec![], binding0: vec![] };
        for (a, b) in zipped {
            let akind = a.kind();
            let bkind = b.kind();
            let aisk = akind.is_keyword();
            let aisp = akind.is_punct();
            let bisk = bkind.is_keyword();
            let bisp = bkind.is_punct(); 
            if akind != bkind {
                fail!();
            } else if aisk || aisp || bisk || bisp {
                // if anyone is a keyword, then it
                // either it must be treated with tokenf
                // or fail
                if aisk && bisk || aisp && bisp {
                    (self.tokenf)(a, b, tin);
                } else {
                    fail!();
                }
            } else {
                if !self.workon(a, b, tin) {
                    fail!();
                }

                tin.binding.append(&mut self.loopnodes(a, b).binding);
                //if an error occurs it will propagate
                // Not recreating the list of children
                // because the nodes are modified in place
            }
        }
        return tin;
    }

    fn workon(&self, node1: &'a Snode, node2: &'a mut Rnode, tin: &'a mut Tin) -> bool {
        // Metavar checking will be done inside the match
        // block below
        // to note: node1 and node2 are of the same SyntaxKind
        match node1.wrapper.metavar {
            crate::parsing_cocci::ast0::MetaVar::NoMeta => {
                if node2.children.len() == 0 {//end of node
                    return node1.astnode.to_string() == node2.astnode.to_string() 
                }
                return true;
            },
            crate::parsing_cocci::ast0::MetaVar::Exp(info) => {
                if  node1.wrapper.metavar.getname() == node1.astnode.to_string() { 
                    // this means it is not complex node
                    // A complex node is defined as anything
                    // which is not a single metavariable
                    tin.binding.push((node1, node2));
                }
                return true;
            },
            crate::parsing_cocci::ast0::MetaVar::Id(info) => {
                // since these are already identifiers no
                // extra checks are there
                return 
                        if node1.wrapper.metavar.getname() == node2.astnode.to_string() { 
                            true
                        } 
                        else { 
                            false
                        };
            },
        }
    }   
    
}

// We can use Result Object Error ass error codes when it fails

/*
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
*/


/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {}
