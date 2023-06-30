use coccinelleforrust::{
    commons::util::{getstmtlist, visitrnode, worksnode},
    engine::disjunctions::getdisjunctions,
    engine::{
        cocci_vs_rs::{Looper, MetavarBinding},
        disjunctions::Disjunction,
        transformation::transform,
    },
    parsing_cocci::ast0::Snode,
    parsing_cocci::{ast0::MODKIND, parse_cocci::processcocci},
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
use itertools::Itertools;
use rand::Rng;
use std::fs;
use std::process::Command;

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
    let mut rng = rand::thread_rng();

    let mut rules = processcocci(&patchstring);
    //rules[0].patch.plus.print_tree();

    let rnode = processrs(&rustcode);
    let mut transformedcode = processrs(&rustcode);

    let looper = Looper::new(tokenf);

    let mut a: Disjunction =
        getdisjunctions(Disjunction(vec![getstmtlist(&mut rules[0].patch.minus).clone().children]));

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

    let randfilename = format!("tmp{}.rs", rng.gen::<u32>());
    transformedcode.writetreetofile(&randfilename);
    Command::new("rustfmt")
        .arg("--config-path")
        .arg("src/rustfmt.toml")
        .arg(&randfilename)
        .output()
        .expect("rustfmt failed");

    let data = fs::read_to_string(&randfilename).expect("Unable to read file");
    println!("After Formatting:\n\n{}", data);

    fs::remove_file(&randfilename).expect("No file found.");
    //rules[0].patch.minus.print_tree();
}

fn main() {
    let file = std::env::args().collect_vec()[1..].join(" ");
    let coccifile = String::from(format!("{}.cocci", file));
    let rsfile = String::from(format!("{}.rs", file));
    transformfile(coccifile, rsfile);
}

