use parser::SyntaxKind;

use super::{parse_cocci::Rule, wrap::Rnode};

type Tag = SyntaxKind;
pub fn set_logilines_aux(mut prevline: usize, node: &mut Rnode, mut bnos: usize) -> usize{
    if node.children.len() == 0 {
        // this is only for testing will be removed after enough tests

        let mut start = node.wrapper.getlinenos().0;
        let logicalblanks = node.wrapper.getlinenos().0-bnos;
        if logicalblanks > prevline {
            start+=1;
            bnos += logicalblanks-prevline-1;
        }
        node.wrapper.set_logilines_end(start);
        bnos
    } else {
        node.wrapper.set_logilines_start(prevline);
        for child in &mut node.children {
            child.wrapper.set_logilines_start(prevline);
            bnos = set_logilines_aux(prevline, child, bnos);
            prevline = child.wrapper.getlogilinenos().1;
        }
        node.wrapper.set_logilines_end(prevline);
        bnos
    }
}

pub fn set_logilines(rules: &mut Vec<Rule>) {
    let mut offsetline = 0;
    for rule in rules {
        set_logilines_aux(offsetline, &mut rule.patch.minus, 0);
        offsetline = rule.patch.minus.wrapper.getlogilinenos().1 - 1;
    }
}
