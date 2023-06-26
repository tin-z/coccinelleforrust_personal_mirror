use std::cmp::{max, min};

use crate::{commons::util::workrnode, parsing_rs::ast_rs::Rnode};

use super::cocci_vs_rs::Environment;

pub fn transform(node: &mut Rnode, env: &Environment) {
    let f = &mut |x: &mut Rnode| -> bool {
        let mut shouldgodeeper: bool = false;
        for minus in env.minuses.clone() {
            let pos = x.getpos();

            if pos == minus {
                x.wrapper.isremoved = true;
            } else if max(pos.0, minus.0) <= min(pos.1, minus.1) {
                //this if checks for an overlap between the rnode and all minuses
                //(and pluses too which will be added later)
                shouldgodeeper = true;
                //if there is even one minus which partially
                //overlaps with the node we go deeper
            }
        }
        return shouldgodeeper;
    };
    workrnode(node, f);
}
