use syntax::SyntaxNode;

use crate::wrap::Rnode;

#[macro_export]
macro_rules! syntaxerror {
    ($lino: expr, $err:expr) => {
        panic!("{:?} at line:{:?}",
                 $err,
                 $lino)
    };
    ($lino:expr, $err:expr, $name:expr) => {
        panic!("{:?}: {:?} at line:{:?}",
                $name,
                $err,
                $lino)
    };
    ($err:expr, $name:expr) => {
        panic!("{:?}: {:?}",
                $name,
                $err)
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
    match &mut v[..3] {
        [a, b, c] => [a, b, c],
        _ => {
            panic!("Does not have three elements")
        }
    }
}


pub fn worktree(mut node: Rnode, f: &mut dyn Fn(&mut Rnode)){
    f(&mut node);
    for child in node.children_with_tokens{
        worktree(child, f);
    }
}
