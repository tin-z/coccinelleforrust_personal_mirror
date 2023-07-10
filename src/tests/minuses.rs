#![allow(dead_code)]
use std::{fs};

use crate::{engine::transformation, parsing_rs::{parse_rs::processrs, ast_rs::Rnode}};

static PREFIX: &str = "./src/tests/";
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
    let expected = fs::read_to_string(format!("{}{}", PREFIX, expectedfile)).expect("This should not be empty.");
    let rnode = processrs(&expected).unwrap();
    return rnode.equals(&out);
}

#[test]
pub fn test1() {
    assert!(
        testtransformation("minuses/test1.cocci", "minuses/test1.rs", "minuses/expected1.rs")
    )
}

#[test]
pub fn test2() {
    assert!(
        testtransformation("minuses/test2.cocci", "minuses/test2.rs", "minuses/expected2.rs")
    )
}

#[test]
pub fn test3() {
    assert!(
        testtransformation("minuses/test3.cocci", "minuses/test3.rs", "minuses/expected3.rs")
    )
}
