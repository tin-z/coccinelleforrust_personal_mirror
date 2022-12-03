use std::collections::HashMap;
use std::{fs};
use syntax::ast::*;
use syntax::SourceFile;

#[derive(PartialEq, Eq, Hash, Clone)]
struct S{
    g: std::string::String
}
#[derive(PartialEq, Eq, Hash, Clone)]
struct T{
    a: u32,
    b: S
}

///All functions here are for tests
/// None of them are meant to be implemented

fn main(){
    let mut data: HashMap<T, u32> = HashMap::new();
    let test = T{a:43, b: S{g:"testing".to_owned()}};
    let gg = test.clone();
    data.insert(test, 341);

    println!("{:?}", data.get(&gg));
}

//test
fn maien() {
    let contents = fs::read_to_string("./main.rs")
        .expect("This shouldnt be empty");
    let parse = SourceFile::parse(&contents);
    let rnode = parse.tree();

    rnode.shebang_token();

    //    let token = &file.shebang_token().unwrap();
    let mut lino = 1;

    for item in rnode.syntax().children_with_tokens(){
        println!("{:?}", item.text_range());
        
    }
    

    //    let (gnodes, mut errors) =
}
