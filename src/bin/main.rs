use coccinelleforrust::{wrap::wrap_root};
use std::{fs, path};
use syntax::{ast::{*, make::name}, ted::Element};
use coccinelleforrust::make_parsable::make_parsable;

fn main() {
    let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
        .expect("This shouldnt be empty");
    
    wrap_root(&contents[..]);
}
