use coccinelleforrust::visitor_ast0::{wraproot, Node, Token, Rnode};
use std::fs;
use syntax::{ast::*, ted::Element};
use parser::SyntaxKind::*;


fn printpathname(node: Path){
    let ss = node.syntax().to_string();
    println!("INSIDE PNAME :: {ss}");
    //TODO: take care of segments() scenario
    if node.segment()!=None{
        let name = node.segment().unwrap().name_ref().unwrap().to_string();
        print!("{name} ");
    }
}

fn printmethodname(item : MethodCallExpr)
{
    match item.name_ref(){
        Some(name) => {
            let name = name.to_string();
            print!("{name}")
        }
        None => {}
    }
    
}

fn printiffunc<'a>(item: Rnode<'a>){
    match item.astnode{
        Node(node) => {
            if item.wrapper.has_argslist(){
                if CallExpr::can_cast(node.kind()){
                    let debug = node.to_string();
                    let path = CallExpr::cast(node).unwrap().expr();
                    println!("HEHE = {debug}");
                    printpathname(Path::cast(PathExpr::cast(path.unwrap().syntax().clone()).unwrap().path().unwrap().syntax().clone()).unwrap());
                }
                else if MethodCallExpr::can_cast(node.kind()){
                    printiffunc(MethodCallExpr::cast(node.kind()).unwrap().receiver().unwrap());
                    printmethodname(MethodCallExpr::cast(node).unwrap());
                }
                let lino = item.wrapper.getlineno();
                println!("at line number {lino}");
            }
            else {
                for child in item.children{
                    match child{
                        Some(child) => {
                            printiffunc(child)
                        }
                        None => {}
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
    let hh = wrap.wrapper.getlineno();
    println!("{hh}");
    for item in wrap.children {
        match item {
            Some(item) => match &item.astnode {
                Node(node) => {
                    if Fn::can_cast(node.kind()) {
                        printiffunc(item);
                    }
                }
                Token(token) => {}
            },
            None => {}
        }
    }

    //    let (gnodes, mut errors) =
}
