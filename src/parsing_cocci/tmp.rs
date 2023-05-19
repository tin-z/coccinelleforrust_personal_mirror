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
