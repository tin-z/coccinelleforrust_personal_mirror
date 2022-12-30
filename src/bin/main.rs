use coccinelleforrust::{
    parse_cocci::{processcocci, self},
    wrap::{wrap_root, Rnode},
    logical_lines::set_logilines,
    treeiterator::traveller
};
use std::fs;

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/rr.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let root = wrap_root(contents.as_str());
    for node in traveller::new(&root) {
        println!("{:?}, ", node.kind());
    }
}
