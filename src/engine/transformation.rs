use crate::{parsing_rs::ast_rs::Rnode, commons::util::workrnode};

use super::cocci_vs_rs::Environment;

pub fn transform(node: &mut Rnode, env: &Environment) {
    let f = &mut |x: &mut Rnode| {
        for minus in env.minuses.clone() {
            let pos = x.getpos();

            if pos==minus {
               x.wrapper.isremoved = true;
            }
        }
    };
    workrnode(node, f);
}