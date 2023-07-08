#![allow(dead_code)]
use std::{fs};
use itertools::{izip, Itertools};

use crate::{
    commons::util::{getstmtlist, visitrnode},
    engine::{
        cocci_vs_rs::{Looper, MetavarBinding},
        disjunctions::{getdisjunctions, Disjunction},
    },
    parsing_cocci::{ast0::Snode, parse_cocci::processcocci},
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
fn tokenf<'a>(_node1: &'a Snode, _node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
    // this is
    // Tout will have the generic types in itself
    // ie ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

static PREFIX: &str = "./src/tests";
fn checkbindings(bindings1: Vec<MetavarBinding>, bindings2: Vec<(&str, &str)>) -> bool {
    assert!(bindings1.len() == bindings2.len());
    for binding2 in bindings2 {
        for binding1 in bindings1.clone() {
            if binding2.0 == binding1.metavarinfo.varname {
                return binding2.1 == binding1.rnode.astnode.to_string();
            }
        }
    }
    true
}

fn testfile(cocci: &str, rs: &str, gbindings: Vec<Vec<(&str, &str)>>) {
    let patchstring =
        fs::read_to_string(format!("{}/{}", PREFIX,  cocci)).expect("This shouldnt be empty");
    let rustcode =
        fs::read_to_string(format!("{}/{}", PREFIX, rs)).expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let mut rules = processcocci(&patchstring);
    let rnode = processrs(&rustcode).ok().unwrap();

    let looper = Looper::new(tokenf);
    let a: Disjunction =
        getdisjunctions(Disjunction(vec![getstmtlist(&mut rules[0].patch.minus).clone().children]));
    let envs = visitrnode(&a.0, &rnode, &|a, b| looper.handledisjunctions(a, b, vec![]));
    assert!(envs.len() == gbindings.len());
    let gbindings1 = envs.into_iter().map(|x| x.bindings).collect_vec();
    for (binding1, binding2) in izip!(gbindings1, gbindings) {
        assert!(checkbindings(binding1, binding2));
    }
}

#[test]
pub fn test1() {
    testfile("bindings/test1.cocci", "bindings/test1.rs", vec![
        vec![("e", "force")],
        vec![("e", "force")]
    ]);
}
