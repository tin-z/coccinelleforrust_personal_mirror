use coccinelleforrust::{
    commons::util::{getstmtlist, worktree, visitrnode},
    engine::{ disjunctions::Disjunction, cocci_vs_rs::{Looper, MetavarBinding}},
    engine::disjunctions::{getdisjunctions},
    parsing_cocci::parse_cocci::{self, processcocci},
    parsing_cocci::{
        ast0::{wrap_root, MetaVar, Snode},
        logical_lines::set_logilines,
    },
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
use itertools::enumerate;
use std::{fs, ops::Deref};
use syntax::{AstNode, SourceFile};

fn aux(node: &Snode) {
    if node.wrapper.metavar != MetaVar::NoMeta {
        print!(
            "{} -----------------------------> ",
            node.astnode.to_string()
        );
        println!("{:?}", node.wrapper.metavar);
    } else {
        for child in &node.children {
            aux(&child);
        }
    }
}

fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>>{
    // this is
    // Tout will have the generic types in itself
    // ie  ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let patchstring =
        fs::read_to_string("./src/tests/test5.cocci").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/tests/test5.rs").expect("This shouldnt be empty");

    let mut rules = processcocci(&patchstring);
    let mut rnode = processrs(&rustcode);
    let looper = Looper::new(tokenf);
    //let (g, matched) = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);

    let a: Disjunction = getdisjunctions(Disjunction(vec![getstmtlist(&mut rules[0].patch.plus).clone().children]));
    let envs = visitrnode(&a.0, &rnode, &|k, l| { looper.getbindings(k, l) });
    //println!("{:?}", envs);
    //rnode.displaytree();
    //rnode.print_tree();
    //a.0[0][0].print_tree();
    for env in envs {
        for binding in env.bindings {   
                println!("{} => {}", binding.0.1, binding.1.astnode.to_string());
        }
	println!("New binding");
    }
    println!("{}", a.0[0][0].gettokenstream());
    println!("{}", a.0[0][1].gettokenstream());
    
    println!("{}", a.0[1][0].gettokenstream());
    println!("{}", a.0[1][1].gettokenstream());
    println!("{:?}", a.0[0].len());
    
}
