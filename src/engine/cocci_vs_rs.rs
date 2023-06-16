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

pub type MetavarBinding<'a> = ((String, String), &'a Rnode, bool); //String, String are (rulename, metavarname)

pub struct Tout<'a> {
    failed: bool,
    pub binding: Vec<MetavarBinding<'a>>,
    pub binding0: Vec<MetavarBinding<'a>>,
}

pub struct MetavarBindings<'a> {
    failed: bool,
    pub binding: Vec<Vec<MetavarBinding<'a>>>,
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

    pub fn addbinding(&mut self, mut gbindings: Vec<MetavarBinding<'a>>, binding: MetavarBinding<'a>) {
        gbindings.push(binding);
        self.binding.push(gbindings);
    }

    pub fn new() -> MetavarBindings<'a> {
        MetavarBindings { failed: false, binding: vec![], binding0: vec![] }
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
    bindings1: &'a Vec<&'a MetavarBinding<'a>>,
    bindings2: &'a Vec<MetavarBinding<'a>>,
) -> Vec<&'a MetavarBinding<'a>> {
    bindings1
        .clone()
        .into_iter()
        .chain(bindings2.iter())
        .collect_vec() //passed bindings are chained with the bindings collected
                       //in this match
}

impl<'a> Looper<'a> {
    pub fn new(tokenf: fn(&'a Snode, &'a Rnode) -> Vec<MetavarBinding<'a>>) -> Looper<'a> {
        Looper { tokenf: tokenf }
    }

    //actual matching function. Takes two nodes and recursively matches them
    pub fn matchnodes(
        &'a self,
        node1vec: Vec<&'a Snode>,
        node2vec: Vec<&'a Rnode>,
        bindings: Vec<&MetavarBinding>,
    ) -> MetavarBindings<'a> {
        let mut tin: MetavarBindings = MetavarBindings::new();

        let mut achildren = node1vec.into_iter();
        let mut bchildren = node2vec.into_iter();
        let mut a: &Snode;
        let mut b: &Rnode;
        loop {
            //at first only the first snode child is extracted because
            //if parsedisjs may match multiple nodes so it needs a Vec<Rnode>
            //and if the first element is popped off here, then it needs to be
            //reattached to the vector that is passed to parsedisjs
            match achildren.next() {
                Some(ak) => {
                    a = ak;
                }
                None => {
                    //if it has reached the end of the semantic patch and still not failed
                    //we return the bindings and consider it a success
                    //println!("return binding = {:?}", tin.binding);
                    return tin;
                }
            }

            if a.wrapper.isdisj {
                //if it enters disjunctions its not getting out except by returning
                let tintmpbindings = 
                    if tin.binding.len() != 0 { tin.binding.into_iter().collect_vec() } else { vec![vec![]] };
                let tbindings = tintmpbindings;
                tin = MetavarBindings::new();
                let mut failed: bool = true;
                for tbinding in tbindings {
                    for (i, disj) in enumerate(a.getdisjs()) {
                        let tin_tmp = self.matchnodes(
                            disj.children.iter().chain(achildren.clone()).collect_vec(),
                            bchildren.clone().collect_vec(),
                            combinebindings(&bindings, &tbinding),
                        );
                        if !tin_tmp.failed {
                            for dbinding in tin.binding.iter() {
                                //goes through all the bindings previously aquired from disjunctions

                            }
                            tin.splitbindings(&tbinding, tin_tmp);
                            failed = false;
                        }
                    }
                }

                //if it reached here it means that no disjunctions have matched
                if failed {
                    fail!();
                }
                else {
                    return tin;
                }
            }

            match bchildren.next() {
                Some(bk) => {
                    b = bk;
                }
                None => {
                    //this means semantic patch remains to be matched
                    //but rnodes are finished which results in failure
                    fail!();
                }
            }
            let akind = a.kind();
            let bkind = b.kind();
            let aisk = akind.is_keyword();
            let bisk = bkind.is_keyword();
            if a.kind() != b.kind() &&//the kinds dont match
                a.wrapper.metavar.isnotmeta()
            //It can get away with kinds not matching if a is a metavar
            {
                fail!();
            }
            if aisk || bisk {
                // if anyone is a keyword, then it
                // either it must be treated with tokenf
                // or fail
                if !(aisk && bisk) {
                    fail!()
                }
            } else {
                let tintmpbindings = 
                    if tin.binding.len() != 0 { tin.binding.into_iter().collect_vec() } else { vec![vec![]] };
                tin = MetavarBindings::new();
                //tin.bindings is cleared above
                for tbinding in tintmpbindings {
                    match self.workon(a, b, combinebindings(&bindings, &tbinding)) {
                        //chaining because I need both the previous bindings and the currently matches ones
                        MetavarMatch::Fail => fail!(),
                        MetavarMatch::Maybe(a, b) => {
                            //println!("{} ==== {}", a.astnode.to_string(), b.astnode.to_string());
                            let tin_tmp = self.matchnodes(
                                a.children.iter().collect_vec(),
                                b.children.iter().collect_vec(),
                                combinebindings(&bindings, &tbinding),
                            );
                            //println!("=={}", tin_tmp.binding.len());
                            if !tin_tmp.failed {
                                tin.splitbindings(&tbinding, tin_tmp);
                                //println!("matched big node");
                            } else {
                                fail!();
                            }
                        }
                        MetavarMatch::Match => {
                            //println!("matched little");
                            let minfo = a.wrapper.metavar.getminfo();
                            let binding = ((minfo.0.clone(), minfo.1.clone()), b, true);
                            tin.addbinding(tbinding, binding);
                        }
                        MetavarMatch::Exists => { tin.binding.push(tbinding); }
                    }
                }
            }
        }
    }

    pub fn loopnodes(
        &'a self,
        node1: &'a Snode,
        node2: &'a Rnode,
    ) -> (Vec<Vec<MetavarBinding>>, bool) {
        //this part of the code is for trying to match within a block
        //sometimes the pattern exists a couple children into the tree
        //The only assumption here is that if two statements are in the same block
        //they are siblings
        let mut matched: bool = false;

        let mut bindings: Vec<Vec<MetavarBinding>> = vec![];

        //let mut a: &Snode = node1;
        //let mut b: &Rnode = node2;
        //let mut tin = Tout { failed: false, binding: vec![], binding0: vec![] };
        let achildren = node1.children.iter();
        let mut bchildren = node2.children.iter();

        loop {
            let tin = self.matchnodes(
                achildren.clone().collect_vec(),
                bchildren.clone().collect_vec(),
                vec![],
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
                let (tin_tmp, matched_tmp) = self.loopnodes(node1, b);
                if matched_tmp {
                    matched = matched_tmp;
                }
                bindings.extend(tin_tmp);
            } else {
                break;
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
        bindings: Vec<&MetavarBinding>,
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
    ) -> (Vec<Vec<((String, String), &Rnode)>>, bool) {
        let topbindings = self.matchnodes(node1.children.iter().collect_vec(), vec![node2], vec![]);
        let (mut bindings, matched) = self.loopnodes(node1, node2);
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
