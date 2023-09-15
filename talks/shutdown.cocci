// tokio 47e2ff48d9f1daac7dba9f136b24eed64c87cf40
// illustrates a sequence of statements
// maybe we could inline the e and then use an isomorphism to
// allow e to name any subsequence?
// this would be better with ...

@@
identifier e;
expression rt;
@@

-    let mut e = tokio_executor::enter().unwrap();
-    e.block_on(rt.shutdown_on_idle());
+    rt.shutdown_on_idle();