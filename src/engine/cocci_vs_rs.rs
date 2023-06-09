use std::{vec, option::Iter, slice::IterMut};

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
            println!("mathching kinds: {:?}, {:?}", node1.children.len(), node2.children.len());
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

    pub fn parsedisjs(&'a self, node1: &'a Snode, node2vec: Vec<&'a Rnode>, gbindings: &Vec<MetavarBinding>) -> (Tout, usize) {
        //println!("In disj - {:?}", node2vec.clone().iter().next());
        let mut tin: Tout;
        let mut bindings: Vec<MetavarBinding> = vec![];
        let mut bline: usize;
        for disj in node1.getdisjs() {
            bline = 0;
            let mut n2children = node2vec.iter();
            tin = Tout { failed: false, binding: vec![], binding0: vec![] };
            for a in &disj.children {
                println!("In disj comparing {:?} of kind {:?} - to {:?}", a.astnode.to_string(), a.kind(), node2vec.clone().iter().next());
                if a.wrapper.isdisj {//if there is a disjunction inside a disj
                    let (tin, lstoskip) = self.parsedisjs(a, node2vec.clone(), 
                        &bindings.iter().chain(gbindings.iter()).cloned().collect()
                    );
                    //send it bindings it got from the caller and the bindings it has collected uptil now
                    if tin.failed {//if this fails break from the loop to the nexi disjunction
                        bindings = vec![];
                        break;
                    }
                    //if it passes process how many lines to skip
                    bline += lstoskip;
                    n2children.nth(lstoskip-1);
                }
                if let Some(b) = n2children.next() {
                    //if b sill has children then start matching
                    bline+=1;
                    if a.kind() == b.kind() || (a.isexpr() && b.isexpr()) {

                        let tin_tmp = self.matchnodes(a, b, &bindings);
                        println!("{}", tin_tmp.failed);
                        if !tin_tmp.failed {
                            //if matching succeeds add bindings
                            bindings.extend(tin_tmp.binding);
                        } else {
                            //else fail
                            tin.failed = true;
                            bindings = vec![];
                            break;
                        }
                    }
                    else {
                        tin.failed = true;
                        bindings = vec![];
                        break;
                    }
                }
                else {
                    //if b does not have any children but there still exists nodes to match
                    //the match has failed
                    tin.failed = true;
                    bindings = vec![];
                    break;
                }
            }
            if !tin.failed {
                tin.binding = bindings;
                return (tin, bline);
            }
        }
        tin = Tout {failed: true, binding: vec![], binding0: vec![]};
        return (tin, 0);
    }

    pub fn loopnodes(&'a self, node1: &'a Snode, node2: &'a Rnode) -> Vec<Vec<MetavarBinding>> {
        //this part of the code is for trying to match within a block
        //sometimes the pattern exists a couple children into the tree
        //The only assumption here is that if two statements are in the same block
        //they are siblings
        
        let mut bindings: Vec<Vec<((String, String), &Rnode)>> = vec![];
        
        //let mut a: &Snode = node1;
        //let mut b: &Rnode = node2;
        //let mut tin = Tout { failed: false, binding: vec![], binding0: vec![] };
        let mut achildren = node1.children.iter();
        let mut bchildren = node2.children.iter();
        let mut a: &Snode = achildren.next().unwrap();//a is not an empty semantic patch
        let mut b: &Rnode;
        let mut ismatching:bool = false;
        let mut binding_tmp: Vec<((String, String), &Rnode)> = vec![];

        
        loop {

            if ismatching {
                //only if the snode has started matching should it go
                //to the next node, else if it is not matching a is always
                //loaded with the first element of the semantic patch
                //this reloads the achildren iterator in case it finished
                if let Some(ak) = achildren.next() { 
                    println!("aloha {:?}", ak.kind());
                    a = ak;
                }
                else {
                    //if it reached this far it means the whole semantic node has been matched
                    println!("===============================================================================");
                    bindings.push(binding_tmp);
                    binding_tmp = vec![];
                    
                    achildren = node1.children.iter();
                    a = achildren.next().unwrap();

                    ismatching = false;
                }
                println!("sotti mitthe {:?}", a.kind());
            }

            if a.wrapper.isdisj {
                //this is since IFs are always present under an exprstmt
                println!("In here disj");
                let (tin, ls2skip) = self.parsedisjs(
                    a, 
                    bchildren.clone().collect(), 
                    &binding_tmp.iter().cloned().collect());
                println!("Bindings:- {:?}, skipped - {}", tin.binding, ls2skip);
                if !tin.failed {
                    binding_tmp.extend(tin.binding);
                    ismatching = true;
                    bchildren.nth(ls2skip-1);
                    continue;
                }
                else {
                    binding_tmp = vec![];
                    achildren = node1.children.iter();
                    a = achildren.next().unwrap();
                    ismatching = false;
                }
            }
            
            if let Some(bk) = bchildren.next() {
                b = bk;
                println!("--- {:?}, {:?}, {}", a.kind(), b.kind(), a.kind() == b.kind());
            }
            else {
                break;
            }
            if a.kind() == b.kind() || (a.isexpr() && b.isexpr()) {
                let tin = self.matchnodes(a, b, &binding_tmp);
                println!("SS- {:?}", tin.failed);
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


