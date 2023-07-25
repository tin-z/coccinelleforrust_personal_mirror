@rule2@
expression e2;
identifier i1;
type t1;
@@

fn tcx<'a> (&'a self) -> i1<'tcx> {
    self.tcx
}

@rules3 hastype@
identifier i1;
@@

-i1<'tcx>
+usize