use std::{
    cmp::{max, min},
    collections::HashSet,
    hash::Hash,
};

use itertools::Itertools;

use crate::{
    commons::{
        info::ParseError,
        util::{getstmtlist, visitrnode, workrnode, worksnode},
    },
    engine::cocci_vs_rs::MetavarBinding,
    parsing_cocci::{
        ast0::{Snode, MODKIND},
        parse_cocci::processcocci,
    },
    parsing_rs::{
        ast_rs::{Rnode, Wrap},
        parse_rs::processrs,
    },
};

use super::{
    cocci_vs_rs::{Environment, Looper, MetavarName},
    disjunctions::{getdisjunctions, Disjunction},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConcreteBinding {
    pub metavarinfo: MetavarName,
    pub rnode: Rnode,
}

#[allow(dead_code)]
impl<'a> ConcreteBinding {
    fn new(rname: String, varname: String, rnode: Rnode) -> ConcreteBinding {
        return ConcreteBinding {
            metavarinfo: MetavarName { rulename: rname, varname: varname },
            rnode: rnode,
        };
    }

    fn frommvarbinding(binding: &MetavarBinding) -> ConcreteBinding {
        return ConcreteBinding {
            metavarinfo: binding.metavarinfo.clone(),
            rnode: binding.rnode.clone(),
        };
    }

    pub fn tomvarbinding(&'a self) -> MetavarBinding<'a> {
        return MetavarBinding { metavarinfo: self.metavarinfo.clone(), rnode: &self.rnode };
    }
}

fn duplicaternode(node: &Rnode) -> Rnode {
    let mut rnode =
        Rnode { wrapper: Wrap::dummy(), astnode: node.astnode.clone(), children: vec![] };
    if node.children.len() == 0 {
        return rnode;
    } else {
        for child in &node.children {
            rnode.children.push(duplicaternode(&child));
        }
    }
    return rnode;
}

fn copytornodewithenv(snode: Snode, env: &Environment) -> Rnode {
    if !snode.wrapper.metavar.isnotmeta() {
        if let Some(index) =
            env.bindings.iter().position(|x| x.metavarinfo.varname == snode.astnode.to_string())
        {
            return duplicaternode(env.bindings[index].rnode);
        } else {
            panic!("Metavariable should already be present in environment.");
        }
    }
    let mut rnode = Rnode { wrapper: Wrap::dummy(), astnode: snode.astnode, children: vec![] };
    for child in snode.children {
        rnode.children.push(copytornodewithenv(child, env));
    }
    rnode
}

fn snodetornode(snodes: Vec<Snode>, env: &Environment) -> Vec<Rnode> {
    let mut rnodevec = vec![];
    for snode in snodes {
        rnodevec.push(copytornodewithenv(snode, env));
    }
    rnodevec
}

pub fn transform(node: &mut Rnode, env: &Environment) {
    let findplusses = &mut |x: &mut Rnode| -> bool {
        let mut shouldgodeeper: bool = false;
        let pos = x.getpos();
        for minus in env.modifiers.minuses.clone() {
            if pos == minus || pos.0 >= minus.0 && pos.1 <= minus.1 {
                x.wrapper.isremoved = true;
                shouldgodeeper = true;
            } else if max(pos.0, minus.0) <= min(pos.1, minus.1) {
                //this if checks for an overlap between the rnode and all minuses
                //(and pluses too which will be added)
                shouldgodeeper = true;
                //if there is even one minus which partially
                //overlaps with the node we go deeper
            }
        }
        for (pluspos, pluses) in env.modifiers.pluses.clone() {
            if pos.0 == pluspos && x.children.len() == 0 {
                x.wrapper.plussed.0 = snodetornode(pluses, env);
                //println!("======================== {:?}", x);
            } else if pos.1 == pluspos && x.children.len() == 0 {
                x.wrapper.plussed.1 = snodetornode(pluses, env);
            } else if pluspos >= pos.0 && pluspos <= pos.1 {
                shouldgodeeper = true;
            }
        }
        return shouldgodeeper;
    };
    workrnode(node, findplusses);
}

fn trimpatchbindings(
    patchbindings: &mut Vec<Vec<MetavarBinding>>,
    usedafter: HashSet<MetavarName>,
) {
    for bindings in patchbindings.iter_mut() {
        //this only retains elements which are used later, but this may form duplicares
        bindings.retain(|x| usedafter.contains(&x.metavarinfo));
    }

    let mut tmp = HashSet::new();
    patchbindings.retain(|x| tmp.insert(x.clone()));
    //this line removes duplicates ^
}

pub fn transformfile(patchstring: String, rustcode: String) -> Result<Rnode, ParseError> {
    fn tokenf<'a>(_node1: &'a Snode, _node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
        vec![]
    }

    let rules = processcocci(&patchstring);

    let parsedrnode = processrs(&rustcode);
    let rnode = match parsedrnode {
        Ok(node) => node,
        Err(()) => {
            return Err(ParseError::TARGETERROR);
        }
    };
    //If this passes then The rnode has been parsed successfully
    let mut transformedcode = rnode.clone();

    let mut savedbindings: Vec<Vec<ConcreteBinding>> = vec![vec![]];
    let mut patchbindings: Vec<Vec<MetavarBinding>> = vec![vec![]];
    
    //let rnodes: Vec<Rnode> = vec![];//somewhere to store
    for mut rule in rules {
        let mut a: Disjunction =
            getdisjunctions(Disjunction(vec![getstmtlist(&mut rule.patch.minus).clone().children]));

        for disj in &mut a.0 {
            for node in disj {
                worksnode(node, (), &mut |x: &mut Snode, _| {
                    if x.wrapper.plusesaft.len() != 0 {
                        //println!("{:#?} attached after {}", x.wrapper.plusesaft, x.astnode.to_string());
                    }
                    if x.wrapper.plusesbef.len() != 0 {
                        //println!("{:#?} before {}", x.wrapper.plusesbef, x.astnode.to_string());
                    }
                    if let Some(MODKIND::MINUS) = x.wrapper.modkind {}
                });
            }
        }
        //let metavars = rule.metavars;

        let mut tmpbindings: Vec<Vec<MetavarBinding>> = vec![];
        for bindings in savedbindings.clone() {

            

            if !(rule
                .freevars
                .iter()
                .all(|x| bindings.iter().find(|y| y.metavarinfo == x.getminfo().0).is_some()))
            {
                //if all inherited dependencies of this rule is not satisfied by the bindings then move on
                //to the next bindings
                continue;
            }
            let envs = visitrnode(&a.0, &rnode, &|k, l| {
                let looper = Looper::new(tokenf, &bindings);
                looper.handledisjunctions(k, l)
            }
            );

            for env in envs.clone() {
                transform(&mut transformedcode, &env);
                tmpbindings.push(env.bindings.clone());
            }
        }
        trimpatchbindings(&mut tmpbindings, rule.usedafter);
        //patchbindings.extend(tmpbindings);

        savedbindings.extend(
            tmpbindings
                .into_iter()
                .map(|x| x.into_iter().map(|y| ConcreteBinding::frommvarbinding(&y)).collect_vec())
                .collect_vec(),
        );
        //removes unneeded and duplicate bindings
    }

    return Ok(transformedcode);
}
