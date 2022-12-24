@ rule32 @
expression e;
identifier fun;
@@
-fun(e);
+fun(e, e);



@ rule13 @
expression e;
identifier fun;
@@

-fun(e);
+fun(e, e);


@ rule1 @
expression e;
identifier fun;
@@



-fun(e);
+fun(e, e);

@ rule2 depends on !rule1 && ( rule13     || rule32) @
@@

-gh();
+testing(89);