@rule2@
expression e2;
identifier i1;
type t1;
@@

fn tcx<'a> (&'a self) -> e2<'tcx> {
    self.tcx
}

@rule3@
identifier rule2.e2;
identifier i;
@@

-fn i<'a>(&'a self) -> e2<'tcx> {
+fn hello() -> i1 {
    self.tcx
}