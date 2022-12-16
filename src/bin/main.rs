use coccinelleforrust::{visitor_ast0::{wraproot}, test_exps::{visit_node, visit_keyword}};
use std::{fs, path};
use syntax::{ast::{*, make::name}, ted::Element};
use parser::SyntaxKind::*;

fn main() {
    let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
        .expect("This shouldnt be empty");
    
    let wrap = wraproot(&contents[..], visit_node, visit_keyword);
}
