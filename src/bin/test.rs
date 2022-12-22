@ rule1 @
expression e;
identifier fun;
@@

-fun(e);
+fun(e, e);

@ rule2 depends on rule1 @
@@

-gh();