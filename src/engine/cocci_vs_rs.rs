use std::{ vec};

use itertools::{Itertools};
use parser::SyntaxKind;

use crate::{
    fail,
    parsing_cocci::ast0::{Snode},
    parsing_cocci::ast0::{MetaVar, MODKIND},
    parsing_rs::ast_rs::Rnode,
};

#[derive(Clone, Debug)]
pub struct MetavarName {
    pub rulename: String,
    pub varname: String,
}

#[derive(Clone, Debug)]
pub struct MetavarBinding<'a> {
    pub metavarinfo: MetavarName,
    pub rnode: &'a Rnode,
}

impl<'a> MetavarBinding<'a> {
    fn new(rname: String, varname: String, rnode: &'a Rnode) -> MetavarBinding<'a> {
        return MetavarBinding {
            metavarinfo: MetavarName {
                rulename: rname,
                varname: varname,
            },
            rnode: rnode,
        };
    }
}

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub failed: bool,
    pub bindings: Vec<MetavarBinding<'a>>,
    pub minuses: Vec<(usize, usize)>,
    pub pluses: Vec<(usize, Vec<Snode>)>,
}

impl<'a> Environment<'a> {
    pub fn add(&mut self, env: Self) {
        for binding in env.bindings {
            if !self
                .bindings
                .iter()
                .any(|x| x.metavarinfo.varname == binding.metavarinfo.varname)
            {
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
        Environment {
            failed: false,
            bindings: vec![],
            minuses: vec![],
            pluses: vec![],
        }
    }
}

enum MetavarMatch<'a, 'b> {
    Fail,
    Maybe(&'b Snode, &'a Rnode),
    Match,
    Exists,
}

pub struct Looper<'a> {
    tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>,
}

impl<'a, 'b> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a> {
        Looper { tokenf }
    }

    //actual matching function. Takes two nodes and recursively matches them

    pub fn matchnodes(
        &self,
        nodevec1: &Vec<&'a Snode>,
        nodevec2: &Vec<&'a Rnode>,
        mut env: Environment<'a>,
    ) -> Environment<'a> {
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
                //println!("fail");
                fail!();
            }

            let akind = a.kind();
            let bkind = b.kind();
            //println!("{:?} ===== {:?}", akind, bkind);
            if akind != bkind && a.wrapper.metavar.isnotmeta() {
                //println!("fail");
                fail!()
            }
            match self.workon(a, b, &env.bindings) {
                MetavarMatch::Fail => {
                    fail!()
                }
                MetavarMatch::Maybe(a, b) => {
                    let renv = self.matchnodes(
                        &a.children.iter().collect_vec(),
                        &b.children.iter().collect_vec(),
                        env.clone(),
                    );
                    if !renv.failed {
                        match a.wrapper.modkind {
                            Some(MODKIND::MINUS) => {
                                env.minuses.push(b.getpos());
                            }
                            _ => {}
                        }
                        if a.wrapper.plusesbef.len() != 0 {
                            env.pluses
                                .push((b.wrapper.info.charstart, a.wrapper.plusesbef.clone()));
                        }
                        if a.wrapper.plusesaft.len() != 0 {
                            env.pluses
                                .push((b.wrapper.info.charend, a.wrapper.plusesaft.clone()));
                        }

                        env.add(renv);
                    } else {
                        fail!()
                    }
                }
                MetavarMatch::Match => {
                    let minfo = a.wrapper.metavar.getminfo();
                    let binding = MetavarBinding::new(minfo.0.clone(), minfo.1.clone(), b);
                    match a.wrapper.modkind {
                        Some(MODKIND::MINUS) => {
                            env.minuses.push(b.getpos());
                        }
                        Some(MODKIND::PLUS) => {}
                        None => {}
                    }

                    if a.wrapper.plusesbef.len() != 0 {
                        env.pluses
                            .push((b.wrapper.info.charstart, a.wrapper.plusesbef.clone()));
                    }
                    if a.wrapper.plusesaft.len() != 0 {
                        env.pluses
                            .push((b.wrapper.info.charend, a.wrapper.plusesaft.clone()));
                    }
                    env.addbinding(binding);
                }
                MetavarMatch::Exists => {
                    if a.wrapper.plusesbef.len() != 0 {
                        env.pluses
                            .push((b.wrapper.info.charstart, a.wrapper.plusesbef.clone()));
                    }
                    if a.wrapper.plusesaft.len() != 0 {
                        env.pluses
                            .push((b.wrapper.info.charend, a.wrapper.plusesaft.clone()));
                    }
                    match a.wrapper.modkind {
                        Some(MODKIND::MINUS) => {
                            env.minuses.push(b.getpos());
                        }
                        Some(MODKIND::PLUS) => {}
                        None => {}
                    }
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
        bindings: &Vec<MetavarBinding>,
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
            metavar => {
                //NOTE THIS TAKES CARE OF EXP AND ID ONLY
                //println!("Found Expr {}, {:?}", node1.wrapper.metavar.getname(), node2.kind());
                if let Some(binding) = bindings
                    .iter()
                    .find(|binding| binding.metavarinfo.varname == node1.wrapper.metavar.getname())
                {
                    if binding.rnode.equals(node2) {
                        MetavarMatch::Exists
                    } else {
                        MetavarMatch::Fail
                    }
                } else {
                    match metavar {
                        MetaVar::Exp(_info) => {
                            if node2.isexpr() {
                                return MetavarMatch::Match;
                            }
                            return MetavarMatch::Fail;
                        }
                        MetaVar::Id(_info) => {
                            if node2.kind() == SyntaxKind::IDENT || node2.ispat() {
                                return MetavarMatch::Match;
                            }
                            return MetavarMatch::Fail;
                        }
                        MetaVar::NoMeta => {
                            panic!("Should never occur");
                            //since no meta has been taken care of in the previous match
                        }
                    }
                }
            }
        }
    }

    pub fn handledisjunctions(
        &'a self,
        disjs: &'a Vec<Vec<Snode>>,
        node2: &Vec<&'a Rnode>,
    ) -> (Vec<Environment>, bool) {
        let mut environments: Vec<Environment> = vec![];
        let mut matched = false;
        for disj in disjs {
            let env = self.matchnodes(&disj.iter().collect_vec(), node2, Environment::new());
            matched = matched || !env.failed;
            if !env.failed {
                environments.push(env);
            }
        }
        (environments, matched)
    }
}
