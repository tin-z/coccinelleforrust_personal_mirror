use std::vec;
use parser::SyntaxKind;
use syntax;
use syntax::SyntaxElement;

type Tag = SyntaxKind;

pub fn work_node<'a, D>(
    do_stuff: &dyn Fn(SyntaxElement, String, &dyn Fn(&SyntaxElement) -> Vec<D>) -> D,
    estrings: String,
    node: SyntaxElement,
) -> D {
    do_stuff(node, estrings.clone(), &|node| -> Vec<D> {
        let mut children = vec![];
        let mut estrings: String = String::new();
        //let mut children = vec![];
        match node {
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens() {
                    match child.kind() {
                        Tag::WHITESPACE | Tag::COMMENT=> {
                            estrings.push_str(child.to_string().as_str());
                        }
                        _ => {
                            children.push(work_node(do_stuff, estrings, child));
                            estrings = String::new();
                        }
                    }
                    //children.push(node);
                }
            }
            SyntaxElement::Token(_token) => {}
        }
        children
    })
}