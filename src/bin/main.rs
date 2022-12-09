use coccinelleforrust::visitor_ast0::{wraproot, Node, Token, Rnode};
use std::{fs, path};
use syntax::{ast::{*, make::name}, ted::Element};
use parser::SyntaxKind::*;


fn printpathname(path: &Rnode){
    //TODO: take care of segments() scenario
    match &path.children[1] {
        Some(segment) => {
            let nameref = segment.children[1].as_ref().unwrap();//Path segment MUST have name
            match &nameref.astnode{
                Node(name) => {
                    let name = name.to_string();
                    let lino = path.wrapper.getlineno();
                    println!("{name} at line number {lino}")
                }
                Token(token) => {}
            }
        }
        None => {}
    }
}

fn printmethodname(item : &Rnode)//MethodCallExpr
{
    let name = item.astnode.to_string();
    let lino = item.wrapper.getlineno();
    println!("{name} at line number {lino}");
}

fn printiffunc<'a>(item: &Rnode<'a>){
    match &item.astnode{
        Node(node) => {
            match (node.kind(), [&item.children.get(0), &item.children.get(2)]){//0 is the path expression
                (CALL_EXPR, [Some(Some(expr)), _]) => {//ask madam about Some(Some())
                    printiffunc(expr);
                }
                (METHOD_CALL_EXPR, [ Some(Some(receiver)), Some(Some(name_ref)) ] ) => {
                    printiffunc(receiver);
                    printmethodname(name_ref);
                }
                (PATH_EXPR, [Some(Some(path)), _])=> {
                    printpathname(path)
                }
                _ => { 
                    for child in &item.children{
                        match child{
                            Some(child) => { printiffunc(&child); }
                            None => {}
                        }                        
                    }
                 }
            }
        }
        _ => {}
    }
}

fn main() {
    let contents = fs::read_to_string("./src/rust-analyzer/crates/ide-db/src/items_locator.rs")
        .expect("This shouldnt be empty");
    let wrap = wraproot(&contents[..]).unwrap();
    for item in wrap.children {
        match item {
            Some(item) => match &item.astnode {
                Node(node) => {
                    match node.kind(){
                        FN => {
                            printiffunc(&item);
                        }
                        _ => {}
                    }
                }
                Token(token) => {}
            },
            None => {}
        }
    }

    //    let (gnodes, mut errors) =
}
