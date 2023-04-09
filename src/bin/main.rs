use coccinelleforrust::{
    parsing_cocci::parse_cocci::{processcocci, self},
    parsing_cocci::{ast0::{wrap_root, Snode, MetaVar}, logical_lines::set_logilines}, 
    parsing_rs::{parse_rs::processrs, ast_rs::Rnode}, 
    engine::cocci_vs_rs::{Tin, MetavarBinding, Looper},
};
use std::fs;

fn aux(node: &Snode){
    if node.wrapper.metavar != MetaVar::NoMeta{
        print!("{} -----------------------------> ", node.astnode.to_string());
        println!("{:?}", node.wrapper.metavar);
    }
    else{
        for child in &node.children{
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
    let patchstring = fs::read_to_string("./src/bin/test2.rs").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/bin/test3.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let mut rules = processcocci(&patchstring);
    let mut rnode = processrs(&rustcode);

    //rules[0].patch.plus.print_tree();
    rnode.print_tree();
    let looper = Looper::new(tokenf);
    let g = looper.loopnodes(&rules[0].patch.plus, &rnode);
    println!("{:?}", g.binding);
}
