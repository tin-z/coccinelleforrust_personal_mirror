use parser::SyntaxKind;

use crate::{wrap::Rnode, parse_cocci::rule};

type Tag = SyntaxKind;
pub fn set_logilines_aux(mut prevline: usize, node: &mut Rnode){
    if node.children_with_tokens.len()==0{
        // this is only for testing will be removed after enough tests
        if node.kind() != Tag::WHITESPACE
        {
            assert!(node.astnode.to_string().matches('\n').count()==0)
        }

        
        let mut start = node.wrapper.getlogilinenos().0;
        if node.astnode.to_string().matches('\n').count()>0 {
        // if node has no children but still has a newline, it means it
        // must  be a whitespace
             start+=1;
        }
        if node.kind()==Tag::IDENT{
            //println!("start {start}")
        }
        node.wrapper.set_logilines_end(start);
    }
    else
    {
        let tmp = prevline;
        for child in &mut node.children_with_tokens{
            let jj = prevline;
            child.wrapper.set_logilines_start(prevline);
            set_logilines_aux(prevline, child);
            prevline=child.wrapper.getlogilinenos().1;
            //println!("{} - KIND - {:?}:{}, {}", child.astnode.to_string(), child.kind(), jj, prevline);
        }
        node.wrapper.set_logilines_start(tmp);
        node.wrapper.set_logilines_end(prevline);
    }
}

pub fn set_logilines(rules: &mut Vec<rule>){
    let mut offsetline = 0;
    for rule in rules{
        //println!("{:?}", rule.patch.minus.children_with_tokens[0].wrapper.getlogilinenos());

        println!("RULE {} - {}", rule.name, offsetline);
        set_logilines_aux(offsetline, &mut rule.patch.minus);
        offsetline = rule.patch.minus.wrapper.getlogilinenos().1 - 1;//going to next line

    }
}