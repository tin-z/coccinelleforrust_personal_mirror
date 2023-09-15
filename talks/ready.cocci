// tokio 83273b8b50fd4e7a68c83f9086e2f1bd513174d7
// illustrates multiple rules
// not sure if the use rule works
// do we need the last rule? does it need the trait declaration?

@has_ready@
expression e;
@@

ready!(e)

@depends on has_ready@
@@

+use futures_core::ready;
 use...;

@@
identifier e, t;
@@

-macro_rules! ready {
-    ($e:expr) => {
-        match $e {
-            ::std::task::Poll::Ready(t) => t,
-            ::std::task::Poll::Pending => return ::std::task::Poll::Pending,
-        }
-    };
-}

@@
identifier e, t;
@@

-#[macro_export]
-macro_rules! ready {
-    ($e:expr) => {{
-        use std::task::Poll::{Pending, Ready};
-
-        match $e {
-            Ready(t) => t,
-            Pending => return Pending,
-        }
-    }};
-}
