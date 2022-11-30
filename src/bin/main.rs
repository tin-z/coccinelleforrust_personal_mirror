use std::{fs};
use syntax::ast::*;
use syntax::SourceFile;

//test
fn main() {
    let contents = fs::read_to_string("./test.rs")
        .expect("This shouldnt be empty");
    let parse = SourceFile::parse(&contents);
    let rnode = parse.tree();

    rnode.shebang_token();

    //    let token = &file.shebang_token().unwrap();
    let mut lino = 1;

    for item in rnode.syntax().children_with_tokens(){
        
        match item.as_node(){
        
            Some(node) => { 
                if Fn::can_cast(node.kind()) {
                    let tmp = Fn::cast(node.clone()).unwrap();
                    let name = tmp.name().unwrap();
                    println!("{name} at {lino}");
                }
                lino += node.to_string().matches('\n').count();
                //print!("node - {node}");
         },
            None => {},
        }
        match item.as_token()
        {
            Some(token) => {lino += token.to_string().matches('\n').count(); },
            None => {}
        }
        
    }
    

    //    let (gnodes, mut errors) =
}
