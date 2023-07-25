@rule2@
expression e1, e2;
@@

-e1.type_of(e2)
+e1.bound_type_of(e2).subst_identity()