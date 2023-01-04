use std::vec;
use parser::SyntaxKind;
use syntax;
use syntax::SyntaxElement;

type Tag = SyntaxKind;

pub fn work_node<'a, D>(
    do_stuff: &dyn Fn(SyntaxElement, &dyn Fn(&SyntaxElement) -> Vec<D>) -> D,
    node: SyntaxElement,
) -> D {
    do_stuff(node, &|node| -> Vec<D> {
        let mut children = vec![];
        //let mut children = vec![];
        match node {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    if node.kind() != Tag::WHITESPACE {
                        children.push(work_node(do_stuff, child));
                    }
                    //children.push(node);
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        children
    })
}
