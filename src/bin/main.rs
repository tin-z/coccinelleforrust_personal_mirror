use coccinelleforrust::{
    commons::util::{getstmtlist, visitrnode, worksnode, worktree, worktreernode},
    engine::disjunctions::getdisjunctions,
    engine::{
        cocci_vs_rs::{Looper, MetavarBinding},
        disjunctions::Disjunction,
        transformation::transform,
    },
    parsing_cocci::{parse_cocci::{self, processcocci}, ast0::MODKIND},
    parsing_cocci::{
        ast0::{wrap_root, MetaVar, Snode},
        logical_lines::set_logilines,
    },
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
use ide_db::line_index::WideEncoding;
use itertools::{enumerate, Itertools};
use std::{ascii::escape_default, fmt::format, fs, ops::Deref};
use syntax::{AstNode, SourceFile};

fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
    // this is
    // Tout will have the generic types in itself
    // ie  ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn transformfile(coccifile: String, rsfile: String) {
    let patchstring = fs::read_to_string(coccifile).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(rsfile).expect("This shouldnt be empty");

    let mut rules = processcocci(&patchstring);
    let rnode = processrs(&rustcode);
    let mut transformedcode = processrs(&rustcode);

    let looper = Looper::new(tokenf);

    let a: Disjunction = getdisjunctions(Disjunction(vec![
        getstmtlist(&mut rules[0].patch.minus).clone().children,
    ]));
    let envs = visitrnode(&a.0, &rnode, &|k, l| looper.handledisjunctions(k, l));

    for env in envs {
        println!("Bindings:- \n");
        for binding in env.bindings.clone() {
            println!(
                "{} => {}",
                binding.metavarinfo.varname,
                binding.rnode.astnode.to_string()
            );
        }
        //println!("New binding");
        transform(&mut transformedcode, &env);
    }

    println!("\n\nTransformed Code - \n");
    transformedcode.displaytree();
    println!();

    //rules[0].patch.minus.print_tree();
}

fn mains() {
    let file = std::env::args().collect_vec()[1..].join(" ");
    let coccifile = String::from(format!("{}.cocci", file));
    let rsfile = String::from(format!("{}.rs", file));
    transformfile(coccifile, rsfile);
}

fn main() {
    let coccifile = String::from("./src/tests/pluses/test1.cocci");
    let patchstring = fs::read_to_string(coccifile).expect("This shouldnt be empty");
    let mut rules = processcocci(&patchstring);
    worksnode(&mut rules[0].patch.minus, (), &mut |x: &mut Snode, _| {
        if x.wrapper.plusesaft.len() != 0 {
            println!("{:#?} attahced after {}", x.wrapper.plusesaft, x.astnode.to_string());
        }
        if x.wrapper.plusesbef.len() != 0 {
            println!("{:#?} before {}", x.wrapper.plusesbef, x.astnode.to_string());
        }
        if let Some(MODKIND::MINUS) = x.wrapper.modkind {
            println!("hello");
        }
    });
    println!("{}", rules[0].patch.minus.astnode.to_string());
    println!("{}", rules[0].patch.plus.astnode.to_string());
}
