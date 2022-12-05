use std::{fs};
use syntax::ast::*;
use coccinelleforrust::visitor_ast0::{wraproot, Node, Token};

fn main() {
    let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
        .expect("This shouldnt be empty");
        let wrap = wraproot(&contents[..]).unwrap();
        let hh = wrap.wrapper.getlineno();
        println!("{hh}");
        for item in wrap.children{
            match item{
                Some(item) => {
                    match item.astnode{
                        Node(node) => {
                            if Fn::can_cast(node.kind()){
                                let name = Fn::cast(node.clone()).unwrap().name().unwrap();
                                let lino = item.wrapper.getlineno();
                                println!("{name} at {lino}");
                            }
                        }
                        Token(token) => {

                        }
                    }
                }
                None => {}
            }
        }


    //    let (gnodes, mut errors) =
}