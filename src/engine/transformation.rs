use std::{
    cmp::{max, min},
    collections::HashSet,
    hash::Hash,
};

use itertools::Itertools;

use crate::{
    commons::{
        info::ParseError,
        util::{getstmtlist, visitrnode, workrnode},
    },
    engine::cocci_vs_rs::MetavarBinding,
    parsing_cocci::{
        ast0::{MetavarName, Snode},
        parse_cocci::processcocci,
    },
    parsing_rs::{
        ast_rs::{Rnode, Wrap},
        parse_rs::processrs,
    },
};

use super::{
    cocci_vs_rs::{Environment, Looper},
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

fn copytornodewithenv(
    snode: Snode,
    env: &Environment,
    inheritedbindings: &Vec<ConcreteBinding>,
) -> Rnode {
    if !snode.wrapper.metavar.isnotmeta() {
        let inheritedbindings = inheritedbindings.iter().map(|x| x.tomvarbinding()).collect_vec();
        if let Some(mvar) = env
            .bindings
            .iter()
            .chain(inheritedbindings.iter())
            .find(|x| x.metavarinfo.varname == snode.astnode.to_string())
        {
            return duplicaternode(mvar.rnode);
        } else {
            panic!("Metavariable should already be present in environment.");
        }
    }
    let mut rnode = Rnode { wrapper: Wrap::dummy(), astnode: snode.astnode, children: vec![] };
    for child in snode.children {
        rnode.children.push(copytornodewithenv(child, env, inheritedbindings));
    }
    rnode
}

fn snodetornode(
    snodes: Vec<Snode>,
    env: &Environment,
    inheritedbindings: &Vec<ConcreteBinding>,
) -> Vec<Rnode> {
    let mut rnodevec = vec![];
    for snode in snodes {
        rnodevec.push(copytornodewithenv(snode, env, inheritedbindings));
    }
    rnodevec
}

pub fn transform(node: &mut Rnode, env: &Environment, inheritedbindings: &Vec<ConcreteBinding>) {
    let transformmods = &mut |x: &mut Rnode| -> bool {
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
        for (pluspos, isbef, pluses) in env.modifiers.pluses.clone() {
            if pos.0 == pluspos && x.children.len() == 0 && isbef {
                x.wrapper.plussed.0 = snodetornode(pluses, env, inheritedbindings);
                //println!("======================== {:?}", x);
            } else if pos.1 == pluspos && x.children.len() == 0 && !isbef {
                x.wrapper.plussed.1 = snodetornode(pluses, env, inheritedbindings);
            } else if pluspos >= pos.0 && pluspos <= pos.1 {
                shouldgodeeper = true;
            }
        }
        return shouldgodeeper;
    };
    workrnode(node, transformmods);
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
    let mut rnode = match parsedrnode {
        Ok(node) => node,
        Err(errors) => {
            return Err(ParseError::TARGETERROR(errors, rustcode));
        }
    };
    //If this passes then The rnode has been parsed successfully
    let mut transformedcode = rnode.clone();

    let mut savedbindings: Vec<Vec<ConcreteBinding>> = vec![vec![]];
    for mut rule in rules {
        let a: Disjunction =
            getdisjunctions(Disjunction(vec![getstmtlist(&mut rule.patch.minus).clone().children]));

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
            let looper = Looper::new(tokenf, &bindings);

            let envs = visitrnode(&a.0, &rnode, &|k, l| looper.handledisjunctions(k, l));
            for env in envs.clone() {
                transform(&mut transformedcode, &env, &bindings);
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

        let transformedstring = transformedcode.gettokenstream();
        transformedcode = match processrs(&transformedstring) {
            Ok(node) => node,
            Err(errors) => {
                return Err(ParseError::RULEERROR(rule.name.clone(), errors, transformedstring));
                //this error is thrown if a previous transformation does
                //some weird syntactically wrong transformation
            }
        };
        //TODO this part can be improved. instead of reparsing the whole string
        //we modify rnode.finalizetransformation() such that in addition to doing
        //transformations it also deals with the character positions properly,
        //updating them in the new code for the minuses to work
        rnode = transformedcode.clone();
        //removes unneeded and duplicate bindings
    }

    return Ok(transformedcode);
}
