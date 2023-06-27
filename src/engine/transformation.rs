use std::cmp::{max, min};

use crate::{commons::util::workrnode, parsing_rs::ast_rs::Rnode, parsing_cocci::ast0::Snode};

use super::cocci_vs_rs::Environment;

fn snodetornode(snode: Vec<&Snode>, env: &Environment) -> Vec<Rnode> {
    vec![]
}

pub fn transform(node: &mut Rnode, env: &Environment) {
    let f = &mut |x: &mut Rnode| -> bool {
        let mut shouldgodeeper: bool = false;
        let pos = x.getpos();
        for minus in env.minuses.clone() {

            if pos == minus {
                x.wrapper.isremoved = true;
            } else if max(pos.0, minus.0) <= min(pos.1, minus.1) {
                //this if checks for an overlap between the rnode and all minuses
                //(and pluses too which will be added)
                shouldgodeeper = true;
                //if there is even one minus which partially
                //overlaps with the node we go deeper
            }
        }
        for (pluspos, pluses) in env.pluses.clone() {
            if pos.0 == pluspos {
                x.wrapper.plussed.0 = snodetornode(pluses, env);
            }
            else if pos.1 == pluspos {

            }
            else if pluspos >= pos.0 && pluspos <= pos.1 {
                   shouldgodeeper = true;
            }
        }
        return shouldgodeeper;
    };
    workrnode(node, f);
}
