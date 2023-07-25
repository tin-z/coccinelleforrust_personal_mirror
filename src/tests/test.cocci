@rule2@
expression e2;
identifier i1;
type t1;
@@

fn tcx<'a> (&'a self) -> i1<'gcx, 'tcx> {
    self.tcx
}

@rules3@
type rule2.i1;
@@

-il<'gcx, 'tcx>
+il<'tcx>