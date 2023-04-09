use crate::parsing_cocci::ast0::Snode;

#[macro_export]
macro_rules! fail {
    () => {
        return Tin {
            binding: vec![],
            binding0: vec![]
        };
    }
}


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


pub fn worktree(mut node: &mut Snode, f: &mut dyn FnMut(&mut Snode)){
    //use async function to wrap the for loop
    //for other cases TODO
    f(&mut node);
    for child in &mut node.children{
        worktree(child, f);
    }
}
