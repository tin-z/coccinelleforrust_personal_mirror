@ rule32 @      
expression e1, e2, e3;
identifier i1;
@@

establishconnection();
let force = 
(
e2.getforce()
|
torque*e2
)
;