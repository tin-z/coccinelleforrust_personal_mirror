// tokio e73b8a0cc934cd6daaa7d7af38410b561ea24a2f
// seems trivial

@@
expression e1.e2;
@@

Signal::new(e1
- ,e2
)

@@
expression e;
@@

tokio_signal::ctrl_c(
- e
 )