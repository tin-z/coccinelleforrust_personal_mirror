use coccinelleforrust::{
    commons::util::{getstmtlist, worktree},
    engine::cocci_vs_rs::{Looper, MetavarBinding, Tout},
    parsing_cocci::parse_cocci::{self, processcocci},
    parsing_cocci::{
        ast0::{wrap_root, MetaVar, Snode},
        logical_lines::set_logilines,
    },
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};

use std::fs;

fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
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
    let n = fs::read_dir("src/tests/").unwrap().count() as usize/3;
    for i in 1..n+1 {
        let patchstring = fs::read_to_string(format!("./src/tests/test{}.cocci", i))
            .expect("This shouldnt be empty");
        let rustcode = fs::read_to_string(format!("./src/tests/test{}.rs", i))
            .expect("This shouldnt be empty");
        let expected = fs::read_to_string(format!("./src/tests/expected{}.txt", i))
            .expect("This shouldnt be empty");

        //let mut rules = processcocci(contents.as_str());
        //set_logilines(&mut rules);

        let mut rules = processcocci(&patchstring);
        let mut rnode = processrs(&rustcode);
        //rules[0].patch.plus.print_tree();
        //rnode.print_tree();
        let looper = Looper::new(tokenf);
        let g = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);

        let mut output: String = String::new();
        for binding in g.0 {
            for var in binding {
                output.push_str(
                    format!("{:?} => {:?}\n", var.0 .1, var.1.astnode.to_string()).as_str(),
                );
            }
            output.push('\n');
        }
        //println!("{} ===\n{}", output, expected);
        //println!("Running Test {}.", i);
        assert!(output.trim() == expected.trim());
        println!("Test {} passed.-----------------------------------------------------------------------------------------", i);
        //rules[0].patch.plus.print_tree();

        //worktree(&mut rules[0].patch.plus, &mut |x: &mut Snode | if x.wrapper.isdisj { println!("DISJ --> {:?}", x.getdisjs()) });
        //println!("0000000000000000");
        //rnode.print_tree();
    }
}
