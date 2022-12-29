use coccinelleforrust::{
    parse_cocci::{processcocci, self},
    wrap::wrap_root,
    logical_lines::set_logilines
};
use std::fs;

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/test.rs").expect("This shouldnt be empty");

    let mut rules = processcocci(contents.as_str());
    for rule in &mut rules{
        println!("RULE {} - ", rule.name);
        println!("{}", rule.patch.minus.astnode.to_string());
        println!("{:?}", rule.patch.minus.children_with_tokens[0].wrapper.getlogilinenos());
        set_logilines(0, &mut rule.patch.minus);
        println!("{:?}", rule.patch.minus.children_with_tokens[0].wrapper.getlogilinenos());
        //rule.patch.minus.print_tree(&mut String::from("."));
    }

}
