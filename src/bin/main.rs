use coccinelleforrust::{
    parsing_cocci::parse_cocci::{processcocci, self},
    parsing_cocci::{ast0::{wrap_root, Snode, MetaVar}, logical_lines::set_logilines}, 
    parsing_rs::{parse_rs::processrs, ast_rs::Rnode}, 
    engine::cocci_vs_rs::{Tout, MetavarBinding, Looper}, commons::util::worktree,
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

fn getstmtlist<'a>(node: &'a mut Snode) -> &'a Snode{
    //since the patch is wrapped in a function to be parsed
    //this function extracts the stmtlist inside it and removes the curly
    //braces from the start and end of the block
    let stmtlist = &mut node.children[0].children[3].children[0];
    stmtlist.children.remove(0);
    stmtlist.children.remove(stmtlist.children.len()-1);
    return stmtlist;
}

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let patchstring = fs::read_to_string("./src/tests/test2.cocci").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/tests/test2.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let mut rules = processcocci(&patchstring);
    let mut rnode = processrs(&rustcode);
    //rules[0].patch.plus.print_tree();
    //rnode.print_tree();
    let looper = Looper::new(tokenf);
    let g = looper.getbindings(getstmtlist(&mut rules[0].patch.plus), &rnode);
    
    for binding in g {
        for var in binding {
            println!("{:?} => {:?}", var.0.1, var.1.astnode.to_string());
        }
        println!();
    }
    //rules[0].patch.plus.print_tree();
    //worktree(&mut rules[0].patch.plus, &mut |x: &mut Snode | if x.wrapper.isdisj { println!("{:?}", x.getdisjs()) });
    //rnode.print_tree();
}
