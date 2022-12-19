use std::process::Child;

use crate::util::{tuple_of_2, tuple_of_3};
pub use crate::wrap::visit_keyword;
use crate::wrap::{fill_wrap, wrap, Rnode, Syntax};
use ide_db::line_index::LineIndex;
use parser::SyntaxKind;
use syntax::{ast::AstChildren, AstNode};

impl wrap {
    pub fn set_test_exps(&mut self) {
        self.true_if_test = true;
        self.true_if_test_exp = true;
    }
}

pub fn process_exp(exp: &mut Rnode) {
    exp.wrapper.set_test_exps();
    match exp.astnode.kind() {
        SyntaxKind::PAREN_EXPR => {
            let [_lp, exp, _rp] = tuple_of_3(&mut exp.children);
            process_exp(exp);
        }
        _ => {}
    }
}

pub fn set_test_exps(node: &mut Rnode) {
    match node.astnode.kind() {
        SyntaxKind::IF_EXPR => {
            let [_if, cond] = tuple_of_2(&mut node.children);
            process_exp(cond);
        }
        SyntaxKind::WHILE_EXPR => {
            let [_while, cond] = tuple_of_2(&mut node.children);
            process_exp(cond);
        }
        SyntaxKind::BIN_EXPR => {
            let [lhs, op, rhs] = tuple_of_3(&mut node.children);
            if op.astnode.is_relational() { 
                process_exp(lhs); process_exp(rhs); 
            }
        }
        SyntaxKind::PREFIX_EXPR => {//Have to be sure of this identity TODO
            let [op, exp] = tuple_of_2(&mut node.children);
            if op.astnode.is_relational() {
                process_exp(exp); 
            };
        }
        _ => { }
    }
    for node in &mut node.children {
        set_test_exps(node);
    }
}
