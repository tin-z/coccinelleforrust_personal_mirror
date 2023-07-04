use std::{
    cmp::{max, min},
    process::Child,
};

use crate::{
    commons::util::{workrnode, getstmtlist, worksnode, visitrnode},
    parsing_cocci::{ast0::{Snode, MODKIND}, parse_cocci::processcocci},
    parsing_rs::{ast_rs::{Rnode, Wrap}, parse_rs::processrs}, engine::cocci_vs_rs::MetavarBinding,
};

use super::{cocci_vs_rs::{Environment, Looper}, disjunctions::{Disjunction, getdisjunctions}};

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


pub fn transformfile(patchstring: String, rustcode: String) -> Rnode{
    fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
        // this is
        // Tout will have the generic types in itself
        // ie  ('a * 'b) tout //Ocaml syntax
        // Should I replace Snode and Rnode with generic types?
        // transformation.ml's tokenf
        // info_to_fixpos
        vec![]
    }

    let rules = processcocci(&patchstring);
    //rules[0].patch.plus.print_tree();

    let rnode = processrs(&rustcode);
    let mut transformedcode = processrs(&rustcode);

    for mut rule in rules {

        let looper = Looper::new(tokenf);
        let mut a: Disjunction = getdisjunctions(Disjunction(vec![
            getstmtlist(&mut rule.patch.minus).clone().children,
        ]));

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

        let envs = visitrnode(&a.0, &rnode, &|k, l| looper.handledisjunctions(k, l));

        for env in envs.clone() {
            transform(&mut transformedcode, &env);
        }
    }

    return transformedcode;

}