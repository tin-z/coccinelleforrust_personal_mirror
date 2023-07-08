@ rule1 @
expression expr, exp1, exp2;
@@
exp1();
(
cinder;
expr;
+clearcell();
+exp1;
|
block;
exp2(23);
+steer();
)

@rule2@
expression rule1.expr, rule1.exp2, e;
@@

-clear