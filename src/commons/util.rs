use parser::SyntaxKind;

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

pub fn visitrnode<'a>(nodea: &Vec<Vec<Snode>>, nodeb: &'a Rnode, f: &dyn Fn(&Vec<Vec<Snode>>, &'a Rnode) -> (Vec<Environment<'a>>, bool)) -> Vec<Environment<'a>>{
    //use async function to wrap the for loop
    //for other cases TODO
    let mut environments = vec![];
    let tmp = f(nodea, nodeb);
    //println!("==> {}, {:?}", tmp.0.len(), tmp.0);
    if tmp.1 {
        environments.extend(tmp.0);
    }
    for child in &nodeb.children {
        environments.extend(visitrnode(nodea, child, f));
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

pub fn getstmtlist<'a>(node: &'a mut Snode) -> &'a Snode {
    //since the patch is wrapped in a function to be parsed
    //this function extracts the stmtlist inside it and removes the curly
    //braces from the start and end of the block
    let stmtlist = &mut node.children[0] //function
        .children[3] //blockexpr
        .children[0]; //stmtlist
    stmtlist.children.remove(0);
    stmtlist.children.remove(stmtlist.children.len() - 1);
    return stmtlist;
}
