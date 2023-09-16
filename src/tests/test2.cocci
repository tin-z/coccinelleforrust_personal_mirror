@@
struct .*::Rnode rnode;
@@

-Rc::new(rnode)
+Rc::new(BoundNode::Node(rnode))