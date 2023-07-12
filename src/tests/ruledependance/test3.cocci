@rule1@
expression e1, e2;
@@

funcall();
foobar(e1);

(
foo();
|
-bar(e2);
)

@rule2@
expression rule1.e1, rule1.e2;
@@

if e1 > 0 {
+testing(e2);
}