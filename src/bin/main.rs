use coccinelleforrust::{
    parse_cocci::{processcocci, self},
    wrap::{wrap_root, Rnode},
    logical_lines::set_logilines,
    test_exps::set_test_exps, util::worktree
};
use std::fs;

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/rr.rs").expect("This shouldnt be empty");

    //let mut rules = processcocci(contents.as_str());
    //set_logilines(&mut rules);

    let root = wrap_root(contents.as_str());
    worktree(root, &mut |x| set_test_exps(x));
}
