use coccinelleforrust::{
    parsing_cocci::parse_cocci::{processcocci, self},
    parsing_cocci::{ast0::{wrap_root, Snode, MetaVar}, logical_lines::set_logilines}, 
    parsing_rs::{parse_rs::processrs, ast_rs::Rnode}, 
    engine::cocci_vs_rs::{Tout, MetavarBinding, Looper}, commons::util::{worktree, getstmtlist},
};
use syntax::{SourceFile, AstNode};
use std::{fs, ops::Deref};

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
    let patchstring = fs::read_to_string("./src/tests/test11.cocci").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/tests/test11.rs").expect("This shouldnt be empty");

    let mut rules = processcocci(&patchstring);
    let mut rnode = processrs(&rustcode);
    
    let looper = Looper::new(tokenf);
    let g = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);
    
    if true{
    for binding in g {
        for var in binding {
            println!("{:?} => {:?}", var.0.1, var.1.astnode.to_string());
        }
        println!();
    }}

}
