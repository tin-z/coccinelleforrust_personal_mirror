use coccinelleforrust::{
    wrap::wrap_root,
    make_parsable::make_parsable,
    parse_cocci::parse_cocci
};
use std::fs;

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/test.rs")
        .expect("This shouldnt be empty");

    parse_cocci(&contents);
}
