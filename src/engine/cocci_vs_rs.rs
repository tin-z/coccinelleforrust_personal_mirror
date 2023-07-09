use std::vec;

use itertools::Itertools;
use parser::SyntaxKind;

use crate::{
    fail,
    parsing_cocci::ast0::Snode,
    parsing_cocci::ast0::{MetaVar, MODKIND},
    parsing_rs::ast_rs::Rnode,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MetavarName {
    pub rulename: String,
    pub varname: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MetavarBinding<'a> {
    pub metavarinfo: MetavarName,
    pub rnode: &'a Rnode,
}

impl<'a> MetavarBinding<'a> {
    fn new(rname: String, varname: String, rnode: &'a Rnode) -> MetavarBinding<'a> {
        return MetavarBinding {
            metavarinfo: MetavarName { rulename: rname, varname: varname },
            rnode: rnode,
        };
    }
}

#[derive(Clone, Debug)]
pub struct Modifiers {
    pub minuses: Vec<(usize, usize)>,
    pub pluses: Vec<(usize, Vec<Snode>)>,
}

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub failed: bool,
    pub bindings: Vec<MetavarBinding<'a>>,
    pub modifiers: Modifiers,
}

impl<'a> Environment<'a> {
    pub fn add(&mut self, env: Self) {
        for binding in env.bindings {
            if !self.bindings.iter().any(|x| x.metavarinfo.varname == binding.metavarinfo.varname) {
                self.bindings.push(binding);
            }
        }
        //self.bindings.extend(env.bindings);
        self.modifiers.minuses.extend(env.modifiers.minuses);
        self.modifiers.pluses.extend(env.modifiers.pluses);
    }

    pub fn addbinding(&mut self, binding: MetavarBinding<'a>) {
        self.bindings.push(binding);
    }

    pub fn addbindings(&mut self, bindings: &Vec<MetavarBinding<'a>>) {
        for binding in bindings {
            self.bindings.push(binding.clone());
        }
    }

    pub fn new() -> Environment<'a> {
        Environment {
            failed: false,
            bindings: vec![],
            modifiers: Modifiers { minuses: vec![], pluses: vec![] },
        }
    }
}

enum MetavarMatch<'a, 'b> {
    Fail,
    Maybe(&'b Snode, &'a Rnode),
    Match,
    Exists,
}

fn addplustoenv(a: &Snode, b: &Rnode, env: &mut Environment) {
    if a.wrapper.plusesbef.len() != 0 {
        env.modifiers.pluses.push((b.wrapper.info.charstart, a.wrapper.plusesbef.clone()));
    }
    if a.wrapper.plusesaft.len() != 0 {
        env.modifiers.pluses.push((b.wrapper.info.charend, a.wrapper.plusesaft.clone()));
    }
}

#[allow(dead_code)]
fn getmoddednodes<'a>(nodevec2: &Vec<&'a Rnode>) -> Vec<&'a Rnode> {
    let nodevec2tmp = nodevec2.iter().filter(|x| !x.wrapper.isremoved).map(|x| *x);
    //removes minuses from previous rules
    let mut nodevec2 = vec![];
    for i in nodevec2tmp {
        nodevec2.extend(i.wrapper.plussed.0.iter());
        nodevec2.push(i);
        nodevec2.extend(i.wrapper.plussed.1.iter());
    }
    //adds pluses from previous rules' modifications

    return nodevec2;
}

pub struct Looper<'a> {
    _tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>,
}

impl<'a, 'b> Looper<'a> {
    pub fn new(_tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a> {
        Looper { _tokenf }
    }

    //actual matching function. Takes two nodes and recursively matches them

    pub fn matchnodes(
        &self,
        nodevec1: &Vec<&Snode>,
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
                                env.modifiers.minuses.push(b.getpos());
                            }
                            _ => {}
                        }
                        addplustoenv(a, b, &mut env);

                        env.add(renv);
                    } else {
                        fail!()
                    }
                }
                MetavarMatch::Match => {
                    let minfo = a.wrapper.metavar.getminfo();
                    let binding = MetavarBinding::new(
                        minfo.0.rulename.to_string(),
                        minfo.0.varname.to_string(),
                        b,
                    );
                    match a.wrapper.modkind {
                        Some(MODKIND::MINUS) => {
                            env.modifiers.minuses.push(b.getpos());
                        }
                        Some(MODKIND::PLUS) => {}
                        None => {}
                    }
                    addplustoenv(a, b, &mut env);
                    env.addbinding(binding);
                }
                MetavarMatch::Exists => {
                    addplustoenv(a, b, &mut env);
                    match a.wrapper.modkind {
                        Some(MODKIND::MINUS) => {
                            env.modifiers.minuses.push(b.getpos());
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
                            } else {
                                return MetavarMatch::Maybe(node1, node2);
                            }
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
        disjs: &Vec<Vec<Snode>>,
        node2: &Vec<&'a Rnode>,
        inhertiedbindings: Vec<MetavarBinding<'b>>,
    ) -> (Vec<Environment<'a>>, bool)
    where
        'b: 'a,
    {
        let mut environments: Vec<Environment> = vec![];
        let mut matched = false;
        for disj in disjs {
            let mut inheritedenv = Environment::new();
            inheritedenv.addbindings(&inhertiedbindings);
            let env = self.matchnodes(&disj.iter().collect_vec(), node2, inheritedenv);
            matched = matched || !env.failed;
            if !env.failed {
                environments.push(env);
            }
        }
        (environments, matched)
    }
}
