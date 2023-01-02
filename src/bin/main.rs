use coccinelleforrust::{
    parse_cocci::{processcocci, self},
    wrap::{wrap_root, Rnode, MetaVar},
    logical_lines::set_logilines,
    test_exps::set_test_exps, util::worktree
};
use std::fs;

fn aux(node: &Rnode){
    if node.wrapper.metavar != MetaVar::NoMeta{
        print!("{} -----------------------------> ", node.astnode.to_string());
        println!("{:?}", node.wrapper.metavar);
    }
    else{
        for child in &node.children_with_tokens{
            aux(&child);
        }
    }
}

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/test.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let rules = processcocci(&contents);
    //aux(&rules[0].patch.minus);
    for rule in rules{
        println!("{}, ", rule.patch.minus.astnode.to_string());
    }
}
