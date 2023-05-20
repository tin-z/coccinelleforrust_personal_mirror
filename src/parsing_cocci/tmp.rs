use std::cell::RefCell;
use std::collections::{LinkedList, VecDeque};

struct A{}
struct B{
    pub a: A,
}

impl<'a> B{
    fn gg(&mut self, jj: &mut A){

    }
}

fn tt<'a>(a: A) -> B{
    let mut aa = A{};
    let mut aa2 = A{};
    let mut bb = B{
        a: aa,
    };
    bb.gg(&mut aa2);
    bb
}

struct NoLifetime {}
struct WithLifetime <'a> {
    pub field: &'a i32
}

fn main() {
    let mut some_val = NoLifetime {};
    borrow_mut_function(&mut some_val);
    borrow_mut_function(&mut some_val); // Borrowing as mutable for the second time.

    let num = 5;
    let mut life_val = WithLifetime { field: &num };
    borrow_lifetime(&mut life_val);
    borrow_lifetime(&mut life_val); // Borrowing as mutable for the second time.

    let num_again = borrow_lifetime(&mut life_val); // Borrow, assign lifetime result
    borrow_lifetime(&mut life_val); // Compiler: cannot borrow `life_val` as mutable more than once
}

fn borrow_mut_function(val_in: &mut NoLifetime) -> String {
    "abc".to_string()
}
fn borrow_lifetime<'a>(val_in: &'a mut WithLifetime) -> &'a i32 {
    val_in.field
}

struct C<'a> {
    pub a: &'a i32
}

fn aux<'a>(a: &'a A, l: &'a i32) -> C<'a>{
    C { a:l }
}


fn masin() {
    let mut v = vec![1, 2, 3, 4, 5];
    
    
    let (front, back) = v.split_last_mut().unwrap();
    println!("Front part: {:?}", front);
    
    println!("Vector after push: {:?}", v);
}

use std::rc::Rc;

fn maindd() {
    let v = Rc::new(RefCell::new(VecDeque::from(vec![1, 2, 3, 4, 5])));
    //let g = vec![];

    for _ in 0..10 {
        v.borrow_mut().push_back(6);
        println!("VecDeque after push: {:?}", v.borrow());
        

        //let front = v.borrow().range(0..v.borrow().len() - 1).collect::<Vec<_>>();
        //g.push(front);
    }

    // Print the contents of g
    //for front in &g {
      //  println!("Front: {:?}", front);
    //}
}