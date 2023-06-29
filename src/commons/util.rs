use itertools::Itertools;
use parser::SyntaxKind;
use syntax::{SyntaxElement, SourceFile, SyntaxNode};

use crate::{parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode, engine::cocci_vs_rs::{MetavarBinding, Environment}};

#[macro_export]
macro_rules! fail {
    () => {
        return Environment {
            failed: true, 
            bindings: vec![],
            minuses: vec![],
            pluses: vec![]
        }
    };
}

#[macro_export]
macro_rules! syntaxerror {
    ($lino: expr, $err:expr) => {
        panic!("{:?} at line:{:?}", $err, $lino)
    };
    ($lino:expr, $err:expr, $name:expr) => {
        panic!("{:?}: {:?} at line:{:?}", $name, $err, $lino)
    };
    ($err:expr, $name:expr) => {
        panic!("{:?}: {:?}", $name, $err)
    };
}

pub fn tuple_of_2<T>(v: &mut Vec<T>) -> [&mut T; 2] {
    match &mut v[..2] {
        [a, b] => [a, b],
        _ => {
            panic!("Does not have two elements")
        }
    }
}

pub fn tuple_of_3<T>(v: &mut Vec<T>) -> [&mut T; 3] {
    if v.len() != 3 {
        panic!("Should never occur. Length is - {:?}", v.len())
    }
    match &mut v[..3] {
        [a, b, c] => [a, b, c],
        _ => {
            panic!("Does not have three elements")
        }
    }
}

pub fn tuple_of_maybe_3<T>(v: &mut Vec<T>) -> [&mut T; 3] {
    match &mut v[..3] {
        [a, b, c] => [a, b, c],
        _ => {
            panic!("Does not have three elements")
        }
    }
}

pub fn worktree(mut node: &mut Snode, f: &mut dyn FnMut(&mut Snode)) {
    //use async function to wrap the for loop
    //for other cases TODO
    f(&mut node);
    for child in &mut node.children {
        worktree(child, f);
    }
}

pub fn worktreernode(mut node: &mut Rnode, f: &mut dyn FnMut(&mut Rnode)) {
    //use async function to wrap the for loop
    //for other cases TODO
    f(&mut node);
    for child in &mut node.children {
        worktreernode(child, f);
    }
}

pub fn worksnode<T>(mut node: &mut Snode, t: T, f: &mut dyn FnMut(&mut Snode, T) -> T) -> T {
    //use async function to wrap the for loop
    //for other cases TODO
    let mut t = f(&mut node, t);
    for child in &mut node.children {
        t = worksnode(child, t, f);
    }
    t
}

pub fn workrnode(mut node: &mut Rnode, f: &mut dyn FnMut(&mut Rnode) -> bool){
    //use async function to wrap the for loop
    //for other cases TODO
    let t = f(node);
    if !t { return }
    for child in &mut node.children {
        workrnode(child, f);
    }
}

pub fn visitrnode<'a>(nodea: &'a Vec<Vec<Snode>>, nodeb: &'a Rnode, f: &dyn Fn(&'a Vec<Vec<Snode>>, &Vec<&'a Rnode>) -> (Vec<Environment<'a>>, bool)) -> Vec<Environment<'a>>{
    //use async function to wrap the for loop
    //for other cases TODO
    let mut environments = vec![];
    let tmp = f(nodea, &vec![nodeb]);
        
    if tmp.1 {
        environments.extend(tmp.0);
    }
    let nodebchildren = &mut nodeb.children.iter();
    loop {
        if let Some(child) = nodebchildren.next() {
            environments.extend(visitrnode(nodea, child, f));
            
            let tmp = f(nodea, &nodebchildren.clone().collect_vec());
            
            if tmp.1 {
                environments.extend(tmp.0);
            }
        }
        else {
            break;
        }
        
    }
    return environments;
}


pub fn isexpr(node1: &Snode) -> bool {
    use SyntaxKind::*;

    match node1.kind() {
        TUPLE_EXPR
        | ARRAY_EXPR
        | PAREN_EXPR
        | PATH_EXPR
        | CLOSURE_EXPR
        | IF_EXPR
        | WHILE_EXPR
        | LOOP_EXPR
        | FOR_EXPR
        | CONTINUE_EXPR
        | BREAK_EXPR
        | BLOCK_EXPR
        | RETURN_EXPR
        | YIELD_EXPR
        | LET_EXPR
        | UNDERSCORE_EXPR
        | MACRO_EXPR
        | MATCH_EXPR
        | RECORD_EXPR
        | RECORD_EXPR_FIELD_LIST
        | RECORD_EXPR_FIELD
        | BOX_EXPR
        | CALL_EXPR
        | INDEX_EXPR
        | METHOD_CALL_EXPR
        | FIELD_EXPR
        | AWAIT_EXPR
        | TRY_EXPR
        | CAST_EXPR
        | REF_EXPR
        | PREFIX_EXPR
        | RANGE_EXPR
        | BIN_EXPR
        | EXPR_STMT
        | LITERAL => true,
        _ => false,
    }
}


pub fn removestmtbracesaddpluses<'a>(node: &'a mut Snode) {
    //since the patch is wrapped in a function to be parsed
    //this function extracts the stmtlist inside it and removes the curly
    //braces from the start and end of the block
    let stmtlist = &mut node.children[0] //function
        .children[3] //blockexpr
        .children[0]; //stmtlist
    stmtlist.children.remove(0);
    let tmp = stmtlist.children.remove(stmtlist.children.len() - 1);
    let len = stmtlist.children.len();
    attachback(&mut stmtlist.children[len - 1], tmp.wrapper.plusesbef);
}

pub fn getstmtlist<'a>(node: &'a mut Snode) -> &'a Snode {
    //since the patch is wrapped in a function to be parsed
    //this function extracts the stmtlist inside it and removes the curly
    //braces from the start and end of the block
    let stmtlist = &mut node.children[0] //function
        .children[3] //blockexpr
        .children[0]; //stmtlist
    return stmtlist;
}


pub fn attachfront(node: &mut Snode, plus: Vec<Snode>) {
    if node.children.len() == 0 {
        node.wrapper.plusesbef.extend(plus);
    } else {
        attachfront(&mut node.children[0], plus);
    }
}

pub fn attachback(node: &mut Snode, plus: Vec<Snode>) {
    let len = node.children.len();
    if len == 0 {
        node.wrapper.plusesaft.extend(plus);
    } else {
        //println!("deeper to {:?}", node.children[len - 1].kind());
        attachback(&mut node.children[len - 1], plus);
    }
}
