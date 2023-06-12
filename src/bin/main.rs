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
    let patchstring =
        fs::read_to_string("./src/tests/test12.cocci").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/tests/test12.rs").expect("This shouldnt be empty");

    let mut rules = processcocci(&patchstring);
    let mut rnode = processrs(&rustcode);

    let looper = Looper::new(tokenf);
    let g = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);

    if true {
        for binding in g {
            for var in binding {
                println!("{:?} => {:?}", var.0 .1, var.1.astnode.to_string());
            }
            println!();
        }
    }
}
