use parser::SyntaxKind;

use crate::wrap::Rnode;

type Tag = SyntaxKind;
pub fn set_logilines(mut prevline: usize, node: &mut Rnode){
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
            println!("start {start}")
        }
        node.wrapper.set_logilines_end(start);
    }
    else
    {
        let tmp = prevline;
        for child in &mut node.children_with_tokens{
            let jj = prevline;
            child.wrapper.set_logilines_start(prevline);
            set_logilines(prevline, child);
            prevline=child.wrapper.getlogilinenos().1;
            println!("{} - KIND - {:?}:{}, {}", child.astnode.to_string(), child.kind(), jj, prevline);
        }
        node.wrapper.set_logilines_start(tmp);
        node.wrapper.set_logilines_end(prevline);
    }
}