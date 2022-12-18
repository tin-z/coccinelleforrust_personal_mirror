use std::process::Child;

use ide_db::line_index::{LineIndex};
use parser::SyntaxKind;
use syntax::{AstNode};
use crate::wrap::{Rnode, Syntax, fill_wrap, wrap};
use crate::visitor_ast0::ast0::worker;
pub use crate::wrap::visit_keyword;

impl wrap{
    pub fn set_test_exps(&mut self){
        self.true_if_test = true;
        self.true_if_test_exp = true;
    }
}

pub fn process_exp(exp: &mut Rnode){
    exp.wrapper.set_test_exps();
    match exp.astnode.kind(){
        SyntaxKind::PAREN_EXPR => {
            match &mut exp.children[..3]{
                [_lp, exp, _rp] => {
                    process_exp(exp);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

pub fn set_test_exps(node: &mut Rnode){
    let children = &mut node.children;
    match node.astnode.kind(){
        SyntaxKind::IF_EXPR => {
            match &mut children[..2]{
                [_if, cond] => {
                    process_exp(cond);
                }
                _ => {}
            }
        }
        SyntaxKind::WHILE_EXPR => {
            match &mut children[..2]{
                [_while, cond] => {
                    process_exp(cond);
                }
                _ => {}
            }
        }
        SyntaxKind::BIN_EXPR => {
            match &mut children[..3]{
                [lhs, op, rhs] => {
                    if op.astnode.is_relational() { process_exp(lhs); process_exp(rhs); }
                }
                _ => {}
            }
        }
        SyntaxKind::PREFIX_EXPR//Have to be sure of this identity TODO
            => {
                match &mut children[..2]{
                    [op, exp] => {
                        if op.astnode.is_relational() { process_exp(exp); };
                    }
                    _ => {}
                }
            }
        _ => { }
    }
    for node in children{
        set_test_exps(node);
    }
}