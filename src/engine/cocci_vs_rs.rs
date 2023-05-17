use std::vec;

use itertools::{izip, Itertools};
use parser::SyntaxKind;
use regex::Match;
use syntax::{ast::Meta, TextRange};

use crate::{
    commons::{info::ParseInfo, util::isexpr},
    fail,
    parsing_cocci::ast0::Snode,
    parsing_cocci::ast0::{Fixpos, Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

type Tag = SyntaxKind;
type MatchedNode<'a> = (&'a Snode<'a>, &'a mut Rnode<'a>);
type CheckResult<'a> = Result<MatchedNode<'a>, usize>;
pub type MetavarBinding<'a> = (&'a Snode<'a>, &'a Rnode<'a>);

pub struct Tout<'a> {
    failed: bool,
    pub binding: Vec<MetavarBinding<'a>>,
    pub binding0: Vec<MetavarBinding<'a>>,
}

enum MetavarMatch<'a>{
    Fail,
    Maybe(&'a Snode<'a>, &'a Rnode<'a>),
    Match
}

//type Tout<'a> = Vec<(MatchedNode<'a>, &'a Vec<MetavarBinding<'a>>)>;

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

pub struct Looper<'a> {
    tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>
}

impl<'a> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a>{
        Looper {
            tokenf: tokenf
        }
    }

    pub fn loopnodes(&self, node1: &'a Snode, node2: &'a Rnode) -> Tout {
        // It has to be checked before if these two node tags match
        //println!("{:?}", node1.kind());
        if node1.kind()!=node2.kind() || 
           node1.children.len() != node2.children.len() {
            fail!();
        }
    
        let zipped = izip!(&node1.children, &node2.children);
        let mut tin = Tout { failed: false, binding: vec![], binding0: vec![] };
        for (a, b) in zipped {
            let akind = a.kind();
            let bkind = b.kind();
            let aisk = akind.is_keyword();
            let bisk = bkind.is_keyword();
            if aisk || bisk {
                // if anyone is a keyword, then it
                // either it must be treated with tokenf
                // or fail
                if aisk && bisk {
                    tin.binding.append(&mut (self.tokenf)(a, b));
                } else {
                    fail!();
                }
            } else {
                match self.workon(a, b) {
                    MetavarMatch::Fail => {
                        fail!();
                    },
                    MetavarMatch::Maybe(a, b) => {
                        let tin_tmp = self.loopnodes(a, b);
                        if !tin_tmp.failed {
                            tin.binding.append(&mut self.loopnodes(a, b).binding);
                        }
                        else{
                            return tin_tmp;
                        }
                    },
                    MetavarMatch::Match => {
                        tin.binding.push((a, b));
                    },
                }
                //if an error occurs it will propagate
                // Not recreating the list of children
                // because the nodes are modified in place
            }
        }
        return tin;
    }

    fn workon(&self, node1: &'a Snode, node2: &'a Rnode) -> MetavarMatch<'a> {
        // Metavar checking will be done inside the match
        // block below
        // to note: node1 and node2 are of the same SyntaxKind

        //TODO take care of disjunctions like (2|3) > e1
        //TODO take care of matching bound metavars 
        match node1.wrapper.metavar {
            None => {
                if node2.children.len() == 0 //end of node
                {
                    if node1.astnode.to_string() != node2.astnode.to_string() {
                        //basically checks for tokens
                        return MetavarMatch::Fail;
                    }
                }
                return MetavarMatch::Maybe(node1, node2);//not sure
            },
            Some(crate::parsing_cocci::ast0::MetaVar::Exp(info)) => {
                    // this means it is not complex node
                    // A complex node is defined as anything
                    // which is not a single metavariable
                    return MetavarMatch::Match;
            },
            Some(crate::parsing_cocci::ast0::MetaVar::Id(info)) => {
                // since these are already identifiers no
                // extra checks are there
                if info.1 == node2.astnode.to_string() { 
                    return MetavarMatch::Maybe(node1, node2);//TODO
                } 
                else { 
                    return MetavarMatch::Maybe(node1, node2)// TODO
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
