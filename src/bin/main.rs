use coccinelleforrust::{
    parsing_cocci::parse_cocci::{processcocci, self},
    parsing_cocci::{ast0::{wrap_root, Snode, MetaVar}, logical_lines::set_logilines},
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

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let patchstring = fs::read_to_string("./src/bin/test2.rs").expect("This shouldnt be empty");
    let rustcode = fs::read_to_string("./src/bin/test3.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let mut rules = processcocci(&patchstring);
    

}
