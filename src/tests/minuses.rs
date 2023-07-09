use std::{fs, process::Command};

use crate::engine::transformation;

static PREFIX: &str = "./src/tests/";
fn transformfile(coccifile: &str, rsfile: &str) -> String{
    let patchstring = fs::read_to_string(format!("{}{}", PREFIX, coccifile)).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(format!("{}{}", PREFIX, rsfile)).expect("This shouldnt be empty");
    
    let randfilename = String::from("/tmp/target149.rs");
    let transformedcode = transformation::transformfile(patchstring, rustcode);
    transformedcode.ok().unwrap().writetreetofile(&randfilename);
    let a = Command::new("rustfmt")
        .arg("--config-path")
        .arg("rustfmt.toml")
        .arg(&randfilename)
        .output()
        .expect("rustfmt failed");
    
    let data = fs::read_to_string(&randfilename).expect("Unable to read file");
    return data;
}

#[test]
pub fn test1() {
    let out = transformfile("minuses/test1.cocci", "minuses/test1.rs");
    println!("{}", out);
    assert!(
        out.trim() == "fn main() {}"
    );
}

#[test]
pub fn test2() {
    let out = transformfile("minuses/test2.cocci", "minuses/test2.rs");
    println!("{}", out);
    assert!(
        out.trim() == "fn main() {
            if e > 12 {}
        }"
    );
}

