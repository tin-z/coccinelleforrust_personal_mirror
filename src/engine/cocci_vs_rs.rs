use std::vec;

use itertools::{izip, Itertools};
use parser::SyntaxKind;
use regex::Match;
use syntax::{ast::{Meta, StmtList}, TextRange};

use crate::{
    commons::{info::ParseInfo, util::isexpr},
    fail,
    parsing_cocci::ast0::Snode,
    parsing_cocci::ast0::{Fixpos, Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

type Tag = SyntaxKind;
type MatchedNode<'a> = (&'a Snode, &'a mut Rnode);
type CheckResult<'a> = Result<MatchedNode<'a>, usize>;
pub type MetavarBinding<'a> = ((String, String) , &'a Rnode);//String, String are (rulename, metavarname)

pub struct Tout<'a> {
    failed: bool,
    pub binding: Vec<MetavarBinding<'a>>,
    pub binding0: Vec<MetavarBinding<'a>>,
}

enum MetavarMatch<'a>{
    Fail,
    Maybe(&'a Snode, &'a Rnode),
    Match,
    Exists
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
    tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>,
    
}


fn getstmtlist<'a>(node: &'a Snode) -> &'a Snode{
    return &node.children[0].children[3].children[0]
}

impl<'a> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a>{
        Looper {
            tokenf: tokenf
        }
    }

    pub fn matchnodes(&self, node1: &'a Snode, node2: &'a Rnode, bindings: &Vec<((String, String), &Rnode)>) -> Tout {
        //Given is they have the same SyntaxKind
        let mut tin: Tout = Tout { failed: false, binding: vec![], binding0: vec![] };
        if node1.children.len() != node2.children.len() {
            //I am yet to come across two nodes that can be matched but have different
            //number of children on the same level.
            fail!();
        }

        for (a, b) in izip!(&node1.children, &node2.children) {
            let akind = a.kind();
            let bkind = b.kind();
            let aisk = akind.is_keyword();
            let bisk = bkind.is_keyword();
            if aisk || bisk {
                // if anyone is a keyword, then it
                // either it must be treated with tokenf
                // or fail
                if !(aisk && bisk) { 
                    fail!()
                }
            } else {
                println!("mathching: {:?}, {:?}", akind, bkind);
                match self.workon(a, b, bindings.iter().chain(tin.binding.iter()).collect()) {
                    //chaining because I need both the previous bindings and the currently matches ones
                    MetavarMatch::Fail => fail!(),
                    MetavarMatch::Maybe(a, b) => {
                        let mut tin_tmp = self.matchnodes(a, b, bindings);
                        if !tin_tmp.failed {
                            tin.binding.append(&mut tin_tmp.binding);
                        }
                        else {
                            fail!();
                        }
                    },
                    MetavarMatch::Match => {
                        let minfo = a.wrapper.metavar.getminfo();
                        let binding = ((minfo.0.clone(), minfo.1.clone()), b);
                        tin.binding.push(binding);
                    },
                    MetavarMatch::Exists => { }
                }
            }
        }
        tin
    }

    pub fn loopnodes(&'a self, node1: &'a Snode, node2: &'a Rnode) -> Vec<Vec<((String, String), &'a Rnode)>> {
        //this part of the code is for trying to match within a block
        //sometimes the pattern exists a couple children into the tree
        //The only assumption here is that if two statements are in the same block
        //they are siblings
        
        let mut bindings: Vec<Vec<((String, String), &Rnode)>> = vec![];
        
        //let mut a: &Snode = node1;
        //let mut b: &Rnode = node2;
        //let mut tin = Tout { failed: false, binding: vec![], binding0: vec![] };
        let mut achildren = node1.children.iter();
        let mut a: &Snode = achildren.next().unwrap();//a is not an empty semantic patch
        let mut ismatching:bool = false;
        let mut binding_tmp: Vec<((String, String), &Rnode)> = vec![];

        let mut indisj: bool = false;
        
        for b in &node2.children {//why does children need to be explicitly borrowed
                                          //node2 is already a borrowed variable
            if a.wrapper.isdisj {

            }
            if ismatching {
                if let Some(ak) = achildren.next() { 
                    println!("aloha {:?}", ak.kind());
                    a = ak;
                }
                else {
                    //if it reached this far it means the whole semantic node has been matched
                    bindings.push(binding_tmp);
                    binding_tmp = vec![];
                    
                    achildren = node1.children.iter();
                    a = achildren.next().unwrap();
                }
                println!("sotti mitthe {:?}", a.kind());
            }
            if a.kind() == b.kind() || (a.isexpr() && b.isexpr()){
                let tin = self.matchnodes(a, b, &binding_tmp);
            
                if !tin.failed {
                    binding_tmp.extend(tin.binding);
                    ismatching = true;
                }
                else {
                    binding_tmp = vec![];
                    achildren = node1.children.iter();
                    a = achildren.next().unwrap();
                    ismatching = false;
                }
            }
            else {
                binding_tmp = vec![];
                achildren = node1.children.iter();
                a = achildren.next().unwrap();
                ismatching = false;
            }

            let mut tin_tmp = self.loopnodes(node1, b);
            bindings.append(&mut tin_tmp);
            
            //if an error occurs it will propagate
            // Not recreating the list of children
            // because the nodes are modified in place
            }
        
        bindings
    }

    fn workon(&self, node1: &'a Snode, node2: &'a Rnode, bindings: Vec<&((String, String), &Rnode)>) -> MetavarMatch<'a> {
        // Metavar checking will be done inside the match
        // block below
        // to note: node1 and node2 are of the same SyntaxKind
        match &node1.wrapper.metavar {
            crate::parsing_cocci::ast0::MetaVar::NoMeta => {
                if node2.children.len() == 0 //end of node
                {
                    println!("{:?}========{}", node2.kind(), node2.astnode.to_string());
                    
                    if node1.astnode.to_string() != node2.astnode.to_string() {
                        //basically checks for tokens
                        return MetavarMatch::Fail;
                    }
                    else {
                        return MetavarMatch::Exists;
                    }
                }
                return MetavarMatch::Maybe(node1, node2);//not sure
            },
            crate::parsing_cocci::ast0::MetaVar::Exp(info) => {
                if let Some(binding) = bindings.iter().find(|(a, _)| a.1 == node1.wrapper.metavar.getname() ) {
                    if binding.1.equals(node2) {
                        println!("EQUALLLITTYYY - {}", binding.1.astnode.to_string());
                        MetavarMatch::Exists
                    }
                    else {
                        MetavarMatch::Fail
                    }
                }
                else {
                    return MetavarMatch::Match
                }
            },
            crate::parsing_cocci::ast0::MetaVar::Id(info) => {
                // since these are already identifiers no
                // extra checks are there
                if node1.wrapper.metavar.getname() == node2.astnode.to_string() { 
                    return MetavarMatch::Maybe(node1, node2);//TODO
                } 
                else { 
                    return MetavarMatch::Maybe(node1, node2)// TODO
                };
            },
        }
    }   

    pub fn getbindings(&'a self, node1: &'a Snode, node2: &'a Rnode) -> Vec<Vec<((String, String), &Rnode)>>{
        let mut bindings = self.loopnodes(node1, node2);
        let tin = self.matchnodes(node1, node2, &vec![]);
        if tin.binding.len() != 0 {
            bindings.push(tin.binding);
        }
        bindings
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
