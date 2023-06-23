use std::vec;

use itertools::{enumerate, Itertools};
use parser::SyntaxKind;
use syntax::ast::PathExpr;

use crate::{
    commons::info::ParseInfo,
    fail,
    parsing_cocci::ast0::{fill_wrap, Snode, Wrap},
    parsing_cocci::ast0::{Fixpos, Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

use super::disjunctions::Disjunction;

pub type MetavarName = (String, String);
pub type MetavarBinding<'a> = (MetavarName, &'a Rnode); //(rulename, metavarname), bound Rnode
pub type Environment<'a> = Vec<MetavarBinding<'a>>;
pub struct Envirosnment<'a>(Vec<MetavarBinding<'a>>, Vec<(usize, usize)>, Vec<(&'a Snode, &'a Rnode)>);

pub struct MetavarBindings<'a> {
    failed: bool,
    pub binding: Vec<Environment<'a>>,
    pub binding0: Vec<MetavarBinding<'a>>,
}

impl<'a> MetavarBindings<'a> {
    pub fn splitbindings(&mut self, tbinding: &Vec<MetavarBinding<'a>>, tin: Self) {
        if tin.binding.len() == 0 {
            return;
        }
        for binding in tin.binding.into_iter() {
            let mut tmp = tbinding.clone();
            tmp.extend(binding);
            self.binding.push(tmp);
        }
    }

    pub fn getsplitbindings(
        tbinding: &Vec<MetavarBinding<'a>>,
        tin: Self,
    ) -> Vec<Vec<MetavarBinding<'a>>> {
        if tin.binding.len() == 0 {
            return vec![tbinding.clone()];
        }
        let mut b = vec![];
        for binding in tin.binding.into_iter() {
            let mut tmp = tbinding.clone();
            tmp.extend(binding);
            b.push(tmp);
        }
        return b;
    }

    pub fn addbinding(
        &mut self,
        mut gbindings: Vec<MetavarBinding<'a>>,
        binding: MetavarBinding<'a>,
    ) {
        gbindings.push(binding);
        self.binding.push(gbindings);
    }

    pub fn new() -> MetavarBindings<'a> {
        MetavarBindings {
            failed: false,
            binding: vec![],
            binding0: vec![],
        }
    }
}

enum MetavarMatch<'a> {
    Fail,
    Maybe(&'a Snode, &'a Rnode),
    Match,
    Exists,
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

fn getstmtlist<'a>(node: &'a Snode) -> &'a Snode {
    return &node.children[0].children[3].children[0];
}

fn combinebindings<'a>(
    bindings1: &Vec<MetavarBinding<'a>>,
    bindings2: &Vec<MetavarBinding<'a>>,
) -> Vec<MetavarBinding<'a>> {
    bindings1
        .clone()
        .into_iter()
        .chain(bindings2.into_iter().cloned())
        .collect_vec() //passed bindings are chained with the bindings collected
                       //in this match
}

impl<'a> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a> {
        Looper { tokenf: tokenf }
    }

    pub fn matchnodes(node1: &Vec<Snode>, node2: &Vec<&'a Rnode>) {

    }

    pub fn loopnodes(
        &'a self,
        node1: &Disjunction,
        node2: &Vec<&'a Rnode>,
        gbindings: Vec<MetavarBinding<'a>>,
    ) -> (Vec<Vec<MetavarBinding<'a>>>, bool) {
        //this part of the code is for trying to match within a block
        //sometimes the pattern exists a couple children into the tree
        //The only assumption here is that if two statements are in the same block
        //they are siblings
        let mut matched: bool = false;

        let mut bindings: Vec<Environment> = vec![];

        //let mut a: &Snode = node1;
        //let mut b: &Rnode = node2;
        //let mut tin = Tout { failed: false, binding: vec![], binding0: vec![] };
        let mut achildren = node1.iter();
        let mut bchildren = node2.iter();
        
        for disj in node1.0 {
            loop {
                let tin = self.matchnodes(
                    &disj,
                    bchildren.clone().cloned().collect_vec(),
                    gbindings.clone(),
                );
                //println!("SS- {:?}", tin.failed);
                if !tin.failed {
                    matched = true; //if it matches even once we say that the rule
                                    //has been succesfully matched
                    bindings.extend(tin.binding);
                }

                //if the above doesnt match then extract the node from which it didnt match, and send its
                //children for matching(by calling loopnodes on it). Note that node1 remanins the same, as
                //we want to match the semantic patch
                if let Some(b) = bchildren.next() {
                    let (tin_tmp, matched_tmp) =
                        self.loopnodes(node1, &b.children.iter().collect_vec(), gbindings.clone());
                    if matched_tmp {
                        matched = matched_tmp;
                    }
                    bindings.extend(tin_tmp);
                } else {
                    break;
                }
            }
        }

        (bindings, matched)
    }

    //this function decides if two nodes match, fail or have a chance of matching, without
    //going deeper into the node.
    fn workon(
        &self,
        node1: &'a Snode,
        node2: &'a Rnode,
        bindings: Vec<MetavarBinding>,
    ) -> MetavarMatch<'a> {
        // Metavar checking will be done inside the match
        // block below
        // to note: node1 and node2 are of the same SyntaxKind
        match &node1.wrapper.metavar {
            crate::parsing_cocci::ast0::MetaVar::NoMeta => {
                if node2.children.len() == 0
                //end of node
                {
                    //println!("{:?}========{}", node2.kind(), node2.astnode.to_string());

                    if node1.astnode.to_string() != node2.astnode.to_string() {
                        //basically checks for tokens
                        return MetavarMatch::Fail;
                    } else {
                        return MetavarMatch::Exists;
                    }
                }
                return MetavarMatch::Maybe(node1, node2); //not sure
            }
            crate::parsing_cocci::ast0::MetaVar::Exp(info) => {
                //println!("Found Expr {}, {:?}", node1.wrapper.metavar.getname(), node2.kind());
                if let Some(binding) = bindings
                    .iter()
                    .find(|(a, _)| a.1 == node1.wrapper.metavar.getname())
                {
                    if binding.1.equals(node2) {
                        //binding equals XOR POSITIVE/NEGATIVE binding
                        //println!("EQUALLLITTYYY - {}", binding.1.astnode.to_string());
                        MetavarMatch::Exists
                    } else {
                        MetavarMatch::Fail
                    }
                } else {
                    if node2.isexpr() {
                        //println!("Matched-----> {}, {}", node1.wrapper.metavar.getname(), node2.astnode.to_string());
                        return MetavarMatch::Match;
                    }
                    MetavarMatch::Fail
                }
            }
            crate::parsing_cocci::ast0::MetaVar::Id(info) => {
                //TODO SUPPORT IDENTIFIER PATTERNS
                if let Some(binding) = bindings
                    .iter()
                    .find(|(a, _)| a.1 == node1.wrapper.metavar.getname())
                {
                    if binding.1.equals(node2) {
                        //println!("EQUALLLITTYYY - {}", binding.1.astnode.to_string());
                        MetavarMatch::Exists
                    } else {
                        MetavarMatch::Fail
                    }
                } else {
                    if node2.kind() == SyntaxKind::IDENT || node2.ispat() {
                        //println!("Matched-----> {}, {}", node1.wrapper.metavar.getname(), node2.astnode.to_string());
                        return MetavarMatch::Match;
                    }
                    MetavarMatch::Fail
                }
            }
        }
    }

    pub fn getbindings(
        &'a self,
        node1: &'a Snode,
        node2: &'a Rnode,
    ) -> (Vec<Vec<MetavarBinding>>, bool) {
        let topbindings = self.matchnodes(node1.children.iter().collect_vec(), vec![node2], vec![]);
        let (mut bindings, matched) = self.loopnodes(
            &node1.children.iter().collect_vec(),
            &node2.children.iter().collect_vec(),
            vec![],
        );
        if !topbindings.failed {
            {
                bindings.extend(topbindings.binding);
            }
        }
        (bindings, topbindings.failed || matched)
    }
}

/// Test function
pub fn equal_expr(nodeA: Rnode, nodeB: Rnode) {}
