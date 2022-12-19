
fn work_node<D>(do_stuff: &dyn Fn(SyntaxElement, &dyn Fn() -> Vec<D>) -> D, node: SyntaxElement) -> D{
    do_stuff(node, &|| -> Vec<D>{
        let children = vec![];
        match node{
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens(){
                    children.push(work_node(do_stuff, child));
                }
            }
            SyntaxElement::Token(token) => {
                children.push(do_stuff(SyntaxElement::Token(token), &|| {vec![]}));
            }
        }
        children
    })
}

//for wrapping
fn do_stuff<Rnode>(node: SyntaxElement, df: &dyn Fn() -> Vec<Rnode>) -> Rnode{
    let wrapped = wrap(node, get_type(node));
    let children = df();
    let rnode = Rnode::new(wrapped, node.syntax(), children);
    rnode
}

//for collecting the number of something
fn calc_stuff<u32>(node: SyntaxElement, df: Vec<u32>) -> u32{
    n = numberoftihngs(node);
    let children = df();
    n + addallelems(children)
}