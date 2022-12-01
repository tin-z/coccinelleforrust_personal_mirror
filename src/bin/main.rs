use std::{fs};
use syntax::ast::*;
use syntax::SourceFile;

//test
fn main() {
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
