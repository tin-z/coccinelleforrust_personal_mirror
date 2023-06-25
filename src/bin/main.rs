use coccinelleforrust::{
    commons::util::{getstmtlist, worktree, visitrnode, worktreernode},
    engine::{ disjunctions::Disjunction, cocci_vs_rs::{Looper, MetavarBinding}, transformation::transform},
    engine::disjunctions::{getdisjunctions},
    parsing_cocci::parse_cocci::{self, processcocci},
    parsing_cocci::{
        ast0::{wrap_root, MetaVar, Snode},
        logical_lines::set_logilines,
    },
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
use ide_db::line_index::WideEncoding;
use itertools::{enumerate, Itertools};
use std::{fs, ops::Deref, ascii::escape_default, fmt::format};
use syntax::{AstNode, SourceFile};

fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>>{
    // this is
    // Tout will have the generic types in itself
    // ie  ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn transformfile(coccifile: String, rsfile: String) {
    let patchstring =
        fs::read_to_string(coccifile).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(rsfile).expect("This shouldnt be empty");

    let mut rules = processcocci(&patchstring);
    let rnode = processrs(&rustcode);
    let mut transformedcode = processrs(&rustcode);

    let looper = Looper::new(tokenf);

    let a: Disjunction = getdisjunctions(Disjunction(vec![getstmtlist(&mut rules[0].patch.minus).clone().children]));
    let envs = visitrnode(&a.0, &rnode, &|k, l| { looper.handledisjunctions(k, l) });
    
    for env in envs {
        println!("Bindings:- \n");
        for binding in env.bindings.clone() {   
                println!("{} => {}", binding.metavarinfo.varname, binding.rnode.astnode.to_string());
        }
	    //println!("New binding");
        transform(&mut transformedcode, &env);
    }

    println!("\n\nTransformed Code - \n");
    transformedcode.displaytree();
    println!();
}

fn main() {
    let file = std::env::args().collect_vec()[1..].join(" ");
    let coccifile = String::from(format!("{}.cocci", file));
    let rsfile = String::from(format!("{}.rs", file));
    transformfile(coccifile, rsfile);
}
