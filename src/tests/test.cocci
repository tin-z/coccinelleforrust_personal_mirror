

@rule2@
expression e2;
@@

let b = e2;

@rule3@
expression rule2.e2;
@@

let c = 3;
+func( e2);