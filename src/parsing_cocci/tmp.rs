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

fn aux<'a>(a: &'a A) -> i32{
    2
}


fn test<'a>() -> (Vec<A>, Vec<i32>){
    let mut v:  Vec<A> = vec![];
    let mut vr: Vec<&A> = vec![];
    let mut hh = vec![];

    v.push(A {  });
    v.push(A {  });
    v.push(A {  });

    hh.push(aux(&v.last().unwrap()));
    v.push(A {  });

    return (v, hh);
}