struct A{}
struct B<'a>{
    pub a: A,
    pub b: Option<&'a A>
}

fn tt<'a>(a: A) -> B<'a>{
    let aa = A{};
    let mut bb = B{
        a: aa,
        b: None
    };
    bb.b = Some(&bb.a);
    bb
}