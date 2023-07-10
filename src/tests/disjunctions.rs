#![allow(dead_code)]
use std::fs;

use crate::{
    engine::transformation,
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};

static PREFIX: &str = "./src/tests/disjunctions/";
fn transformfile(coccifile: &str, rsfile: &str) -> Rnode {
    let patchstring =
        fs::read_to_string(format!("{}{}", PREFIX, coccifile)).expect("This shouldnt be empty.");
    let rustcode =
        fs::read_to_string(format!("{}{}", PREFIX, rsfile)).expect("This shouldnt be empty.");

    let transformedcode = transformation::transformfile(patchstring, rustcode).ok().unwrap();
    let rnode = processrs(&transformedcode.gettokenstream()).unwrap();
    return rnode;
}

fn testtransformation(coccifile: &str, rsfile: &str, expectedfile: &str) -> bool {
    let out = transformfile(coccifile, rsfile);
    let expected = fs::read_to_string(format!("{}{}", PREFIX, expectedfile))
        .expect("This should not be empty.");
    let rnode = processrs(&expected).unwrap();
    return rnode.equals(&out);
}

#[test]
pub fn test1() {
    assert!(testtransformation("test1.cocci", "test1.rs", "expected1.rs"))
}

#[test]
pub fn test2() {
    assert!(testtransformation("test2.cocci", "test2.rs", "expected2.rs"))
}

#[test]
pub fn test3() {
    assert!(testtransformation("test3.cocci", "test3.rs", "expected3.rs"))
}

#[test]
pub fn test4() {
    assert!(testtransformation("test4.cocci", "test4.rs", "expected4.rs"))
}
