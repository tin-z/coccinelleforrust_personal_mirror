use coccinelleforrust::{wrap::wrap_root, test_exps::set_test_exps};
use std::{fs};

fn main() {
    let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
        .expect("This shouldnt be empty");
    
    let mut node = wrap_root(&contents[..]);
    set_test_exps(&mut node);
}
