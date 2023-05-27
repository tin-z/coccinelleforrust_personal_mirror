use std::cell::RefCell;
use std::collections::{LinkedList, VecDeque};

struct A{}
struct B{
    pub a: A,
}

struct C<'a>{
    pub a: Option<&'a C<'a>>
}

fn testfn1<'a>(a: &'a A, b: &'a B) -> &'a A {
    return &b.a;
}

fn testfn2(){
    let a = A{};
    
}

impl<'a> B{
    fn gg(&mut self, jj: &mut A){

    }
}

fn tt<'a>() -> (){
    let mut v = vec![];
    v.push(C{a: None});
    v.push(C{a: None});

    let (a, b) = v.split_at_mut(1);
    b.last_mut().unwrap().a = Some(a.last().unwrap());


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

struct mvar{}
struct patch<'a>{
    a: Option<&'a mvar>
}

struct rule<'a>{
    m: mvar,
    pt: patch<'a>,
}

fn mainn<'a>() -> () {
    let m = mvar{};
    let pt = patch{
        a: None
    };

    let mut r =  rule{
        m: mvar {  },
        pt: pt
    };

    r.pt.a = Some(&r.m);

}

fn testt() {
    let mut v = vec![];
    v.push(A{});
    v.push(A{});
    v.push(A{});

    let (a, b) = v.split_at_mut(1);
}


fn fa() {
    let a=1;
    
  

}