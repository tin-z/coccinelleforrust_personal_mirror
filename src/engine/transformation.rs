use std::{
    cmp::{max, min},
    process::Child,
};

use crate::{
    commons::util::workrnode,
    parsing_cocci::ast0::Snode,
    parsing_rs::ast_rs::{Rnode, Wrap},
};

use super::cocci_vs_rs::Environment;

fn duplicaternode(node: &Rnode) -> Rnode {
    let mut rnode = Rnode {
        wrapper: Wrap::dummy(),
        astnode: node.astnode.clone(),
        children: vec![],
    };
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
        if let Some(index) = env
            .bindings
            .iter()
            .position(|x| x.metavarinfo.varname == snode.astnode.to_string())
        {
            return duplicaternode(env.bindings[index].rnode);
        } else {
            panic!("Metavariable should already be present in environment.");
        }
    }
    let mut rnode = Rnode {
        wrapper: Wrap::dummy(),
        astnode: snode.astnode,
        children: vec![],
    };
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
    let f = &mut |x: &mut Rnode| -> bool {
        let mut shouldgodeeper: bool = false;
        let pos = x.getpos();
        for minus in env.minuses.clone() {
            if pos == minus {
                x.wrapper.isremoved = true;
                shouldgodeeper = true;
                println!("removed: {}, {:?}", x.astnode.to_string(), x.kind());
            } else if max(pos.0, minus.0) <= min(pos.1, minus.1) {
                //this if checks for an overlap between the rnode and all minuses
                //(and pluses too which will be added)
                shouldgodeeper = true;
                //if there is even one minus which partially
                //overlaps with the node we go deeper
            }
        }
        for (pluspos, pluses) in env.pluses.clone() {
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
    workrnode(node, f);
}
