// SPDX-License-Identifier: GPL-2.0

use super::ast0::{Snode, Wrap};
use crate::commons::util::{tuple_of_2, tuple_of_3, worktree};
use parser::SyntaxKind;
use syntax::SyntaxElement;

type Tag = SyntaxKind;

impl Wrap {
    pub fn set_test_exps(&mut self) {
        self.true_if_test = true;
        self.true_if_test_exp = true;
    }
}

fn is_relational(node: &SyntaxElement) -> bool {
    match node.kind() {
        Tag::AMP2 | Tag::PIPE2 | Tag::BANG => true, //&& || !
        _ => false,
    }
}

fn process_exp(exp: &mut Snode) {
    exp.wrapper.set_test_exps();
    match exp.astnode.kind() {
        Tag::PAREN_EXPR => {
            let [_lp, exp, _rp] = tuple_of_3(&mut exp.children);
            process_exp(exp);
        }
        _ => {}
    }
}

fn set_test_exps_aux(node: &mut Snode) {
    match node.astnode.kind() {
        Tag::IF_EXPR => {
            let [_if, cond] = tuple_of_2(&mut node.children);
            process_exp(cond);
        }
        Tag::WHILE_EXPR => {
            let [_while, cond] = tuple_of_2(&mut node.children);
            process_exp(cond);
        }
        Tag::BIN_EXPR => {
            let [lhs, op, rhs] = tuple_of_3(&mut node.children);
            if is_relational(&op.astnode) {
                process_exp(lhs);
                process_exp(rhs);
            }
        }
        Tag::PREFIX_EXPR => {
            //Have to be sure of this identity TODO
            let [op, exp] = tuple_of_2(&mut node.children);
            if is_relational(&op.astnode) {
                process_exp(exp);
            };
        }
        _ => {}
    }
}

pub fn set_test_exps(mut root: Snode) {
    worktree(&mut root, &mut |x| set_test_exps_aux(x));
}
