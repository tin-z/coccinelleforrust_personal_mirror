use coccinelleforrust::{visitor_ast0::{wraproot}, wrap::{wrap_node_aux, wrap_keyword_aux}};
use std::{fs, path};
use syntax::{ast::{*, make::name}, ted::Element};
use parser::SyntaxKind::*;

fn main() {
    //let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
    //    .expect("This shouldnt be empty");
    let contents = fs::read_to_string("./src/bin/test.rs")
        .expect("This shouldnt be empty");
    let wrap = wraproot(&contents[..], wrap_node_aux, wrap_keyword_aux);
}
