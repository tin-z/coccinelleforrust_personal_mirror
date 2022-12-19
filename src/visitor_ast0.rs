/// visitor_ast0.rs
/// This file contains code for going over(visitng) the Abstract Syntax Tree
/// and output a vector of a user-defined data structure after computation

pub mod ast0;

use ide_db::line_index::LineCol;
use ide_db::line_index::LineIndex;
use syntax::SyntaxElement;
use std::vec;
use syntax;
use syntax::ast::Expr::*;
use syntax::ast::HasLoopBody;
use syntax::ast::HasName;
use syntax::ast::{AnyHasArgList, AstNode, HasModuleItem, SourceFile, Type};
use syntax::SyntaxToken;

use self::ast0::worker;

pub fn work_node<'a, D>(do_stuff: &dyn Fn(SyntaxElement, &dyn Fn() -> Vec<D>) -> D, node: SyntaxElement) -> D{
    do_stuff(node, &|| -> Vec<D>{
        let children = vec![];
        match node{
            SyntaxElement::Node(node) => {
                for child in node.children_with_tokens(){
                    children.push(work_node(do_stuff, child));
                }
            }
            SyntaxElement::Token(token) => {
                children.push(do_stuff(node, &|| {vec![]}));
            }
        }
        children
    })
}