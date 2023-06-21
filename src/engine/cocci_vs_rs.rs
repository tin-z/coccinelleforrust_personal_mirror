use std::{clone, iter::zip, vec, ops::Deref};

use ide_db::base_db::Env;
use itertools::{enumerate, Itertools};
use parser::SyntaxKind;
use syntax::ast::PathExpr;

use crate::{
    commons::info::ParseInfo,
    fail,
    parsing_cocci::ast0::{fill_wrap, Snode, Wrap},
    parsing_cocci::ast0::{Mcodekind},
    parsing_rs::ast_rs::Rnode,
};

pub type MetavarName = (String, String);
pub type MetavarBinding<'a> = (MetavarName, &'a Rnode); //(rulename, metavarname), bound Rnode

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub failed: bool,
    pub bindings: Vec<MetavarBinding<'a>>,
    pub minuses: Vec<(usize, usize)>,
    pub pluses: Vec<usize>,
}

impl<'a> Environment<'a> {
    pub fn add(&mut self, env: Self) {
        for binding in env.bindings {
            if !self.bindings.iter().any(|x| if x.0.1==binding.0.1 { true } else {false}) {
                self.bindings.push(binding);
            }
        }
        //self.bindings.extend(env.bindings);
        self.minuses.extend(env.minuses);
        self.pluses.extend(env.pluses);
    }

    pub fn addbinding(&mut self, binding: MetavarBinding<'a>) {
        self.bindings.push(binding);
    }

    pub fn new() -> Environment<'a> {
        Environment { failed: false, bindings: vec![], minuses: vec![], pluses: vec![] }
    }
}

#[derive(Clone)]
pub struct MetavarBindings<'a> {
    failed: bool,
    pub binding: Vec<Environment<'a>>,
    pub binding0: Vec<MetavarBinding<'a>>,
    pub minuses: Vec<Vec<&'a Rnode>>,
    pub pluses: Vec<&'a Rnode>//not the actual def of pluses TODO
}

enum MetavarMatch<'a, 'b> {
    Fail,
    Maybe(&'b Snode, &'a Rnode),
    Match,
    Exists,
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

impl<'a, 'b> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a> {
        Looper { tokenf }
    }

    //actual matching function. Takes two nodes and recursively matches them

    pub fn matchnodes(
        &self,
        nodevec1: &Vec<Snode>,
        nodevec2: &Vec<&'a Rnode>,
        mut env: Environment<'a>,
    ) -> Environment<'a> {
        let mut newbindings: Environment = Environment::new();
        let mut nodevec1 = nodevec1.iter();
        let mut nodevec2 = nodevec2.iter();
        let mut a: &Snode;
        let mut b: &Rnode;
        loop {
            
            if let Some(ak) = nodevec1.next() {
                a = ak;
            } else {
                return env;
            }

            if let Some(bk) = nodevec2.next() {
                b = *bk;
            } else {
                println!("fail");
                fail!();
            }

            let akind = a.kind();
            let bkind = b.kind();
            let aisk = akind.is_keyword();
            let bisk = bkind.is_keyword();
            //println!("{:?} ===== {:?}", akind, bkind);
            if akind != bkind && a.wrapper.metavar.isnotmeta() {
                println!("fail");
                fail!()
            }
            if aisk || bisk {
                if !(aisk && bisk) {
                    println!("fail");
                    fail!()
                }
            } else {
                match self.workon(a, b, env.bindings.clone()) {
                    MetavarMatch::Fail => {
                        println!("fail");
                        fail!()
                    }
                    MetavarMatch::Maybe(a, b) => {
                        let renv = self.matchnodes(
                            &a.children,
                            &b.children.iter().collect_vec(),
                            env.clone(),
                        );
                        if !renv.failed {
                            println!("{} matched to {}", a.gettokenstream(), b.astnode.to_string());
                            env.add(renv);
                            //println!("{}", env.bindings.len());
                        } else {
                            println!("fail");
                            fail!()
                        }
                    }
                    MetavarMatch::Match => {
                        let minfo = a.wrapper.metavar.getminfo();
                        let binding = ((minfo.0.clone(), minfo.1.clone()), b);
                        //println!("addding binding => {:?}", binding);
                        newbindings.addbinding(binding.clone());
                        env.addbinding(binding);
                        //println!("{:?}", env.bindings);
                    }
                    MetavarMatch::Exists => {}
                }
            }
        }
    }
    //this function decides if two nodes match, fail or have a chance of matching, without
    //going deeper into the node.
    fn workon(
        &self,
        node1: &'b Snode,
        node2: &'a Rnode,
        bindings: Vec<MetavarBinding>,
    ) -> MetavarMatch<'a, 'b> {
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
        disjs: &Vec<Vec<Snode>>,
        node2: &Vec<&'a Rnode>,
    ) -> (Vec<Environment>, bool) {
        //let topbindings = self.matchnodes(node1.children.iter().collect_vec(), vec![node2], vec![]);
        let mut environments: Vec<Environment> = vec![];
        let mut matched = false;
        for disj in disjs {
            let env = self.matchnodes(disj, node2, Environment::new());
            matched = matched || !env.failed;
            if !env.failed {
                environments.push(env);
                println!("{:?}==============================================================", node2[0].kind());
            }
            
        }
        (environments, matched)
    }
}

