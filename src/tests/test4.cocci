@ rule32 @      
expression e1, e2, e3;
identifier i1;
@@


(
if e1<e3 { 
(
    e2
|
    let g = 9;
)
}
|
let b = e3;
)

let a = e3;