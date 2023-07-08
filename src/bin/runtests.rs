use coccinelleforrust::{
    commons::util::{getstmtlist, visitrnode},
    engine::{disjunctions::{getdisjunctions, Disjunction}, cocci_vs_rs::{Looper, MetavarBinding}},
    parsing_cocci::parse_cocci::{processcocci},
    parsing_cocci::{
        ast0::{ Snode},
    },
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};

use std::fs;

#[allow(dead_code)]
fn tokenf<'a>(_node1: &'a Snode, _node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
    // this is
    // Tout will have the generic types in itself
    // ie ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let n = fs::read_dir("src/tests/bindings/").unwrap().count() as usize/3;
    for i in 1..n+1 {

        println!("here");
        let patchstring = fs::read_to_string(format!("./src/tests/bindings/test{}.cocci", i))
            .expect("This shouldnt be empty");
        let rustcode = fs::read_to_string(format!("./src/tests/bindings/test{}.rs", i))
            .expect("This shouldnt be empty");
        let expected = fs::read_to_string(format!("./src/tests/bindings/expected{}.txt", i))
            .expect("This shouldnt be empty");

            println!("here");
        //let mut rules = processcocci(contents.as_str());
        //set_logilines(&mut rules);

        let mut rules = processcocci(&patchstring);
        println!("here");
        let rnode = processrs(&rustcode).ok().unwrap();
        //rules[0].patch.plus.print_tree();
        //rnode.print_tree();
        let looper = Looper::new(tokenf);
    //let (g, matched) = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);
        let a: Disjunction = getdisjunctions(Disjunction(vec![getstmtlist(&mut rules[0].patch.minus).clone().children]));
        let envs = visitrnode(&a.0, &rnode, &|a, b| { looper.handledisjunctions(a, b, vec![]) });
        let mut output: String = String::new();
        for env in envs {
            for var in env.bindings {
                output.push_str(
                    format!("{:?} => {:?}\n", var.metavarinfo.varname, var.rnode.astnode.to_string()).as_str(),
                );
            }
            output.push('\n');
        }

        println!("{} ===\n{}", output, expected);
        //println!("Running Test {}.", i);
        if output.trim() == expected.trim() {
            println!("Test {} passed.-----------------------------------------------------------------------------------------", i);
        }
        else {
            println!("output: {}", output.trim());
            println!("expected: {}", expected.trim());
        }
        
        //rules[0].patch.plus.print_tree();

        //worktree(&mut rules[0].patch.plus, &mut |x: &mut Snode | if x.wrapper.isdisj { println!("DISJ --> {:?}", x.getdisjs()) });
        //println!("0000000000000000");
        //rnode.print_tree();
    }
}
