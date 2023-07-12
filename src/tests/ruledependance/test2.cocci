@rule1@
expression e1;
@@

funcall();
foobar(e1);

@rule2@
expression rule1.e1;
@@

if e1 > 0 {
+testing();
}

@rule3@
@@

-testing();