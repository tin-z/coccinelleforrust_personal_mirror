use syntax::SyntaxTreeBuilder;

use crate::wrap::Rnode;

pub struct traveller<'a> {
    vec: Vec<&'a Rnode>
}

impl<'a> traveller<'a> {

    fn buildtree(mut self, node: &'a Rnode) -> Vec<&'a Rnode>{
        self.vec.push(node);
        for child in &node.children_with_tokens{
            let mut tmp = traveller::new(child);
            self.vec.append(&mut tmp);
        }
        self.vec
    }

    pub fn new(node: &'a Rnode) -> Vec<&'a Rnode> {
        let mut t = traveller {
            vec: vec![node]
        };
        t.buildtree(node)
    }
}
