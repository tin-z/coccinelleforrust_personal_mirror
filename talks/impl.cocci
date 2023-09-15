// tokio commit 474befd23c368a34a5f45aab0f3945212109a803

@@
identifier f, P, p;
type T1, T2;
@@

- f<P: T1>(p: P) -> T2
+ f(p: impl T1) -> T2
{ ... }

@@
identifier f, P, p;
type T1, T2;
@@

- f<P>(p: P) -> T2 where P: T1,
+ f(p: impl T1) -> T2
{ ... }

@@
identifier f, P, Q, p, q;
type T1, T2, T3;
@@

- f<P: T1, Q: T2>(p: P, q: Q) -> T3
+ f(p: impl T1, q: impl T2) -> T3
{ ... }

@@
identifier f, P, Q, p, q;
type T1, T2, T3;
@@

- f<P: T1>(p: P, q: T2) -> T3
+ f(p: impl T1, q: T2) -> T3
{ ... }

@@
identifier f, P, Q, p, q;
type T1, T2, T3;
@@

- f<P: T1>(q: T2, p: P) -> T3
+ f(q: T2, p: impl T1) -> T3
{ ... }

// Needs:
// ... in parameter lists
// parameter metavariable
