@ rule1 @
expression expr, exp1, exp2;
@@

bar();
expr;
want();

@rule2@
expression rule1.expr;
@@

-want();
+want(expr);

@rule3@
@@

-want(hallo());