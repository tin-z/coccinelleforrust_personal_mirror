use parser::SyntaxKind;

use crate::{parse_cocci::rule, wrap::Rnode};

type Tag = SyntaxKind;
pub fn set_logilines_aux(mut prevline: usize, node: &mut Rnode) {
    if node.children_with_tokens.len() == 0 {
        // this is only for testing will be removed after enough tests
        if node.kind() != Tag::WHITESPACE {
            assert!(node.astnode.to_string().matches('\n').count() == 0)
        }

        let mut start = node.wrapper.getlogilinenos().0;
        if node.astnode.to_string().matches('\n').count() > 0 {
            // if node has no children but still has a newline, it means it
            // must  be a whitespace
            start += 1;
        }
        node.wrapper.set_logilines_end(start);
    } else {
        node.wrapper.set_logilines_start(prevline);
        for child in &mut node.children_with_tokens {
            child.wrapper.set_logilines_start(prevline);
            set_logilines_aux(prevline, child);
            prevline = child.wrapper.getlogilinenos().1;
        }
        node.wrapper.set_logilines_end(prevline);
    }
}

pub fn set_logilines(rules: &mut Vec<rule>) {
    let mut offsetline = 0;
    for rule in rules {
        set_logilines_aux(offsetline, &mut rule.patch.minus);
        offsetline = rule.patch.minus.wrapper.getlogilinenos().1 - 1; //going to next line
    }
}
