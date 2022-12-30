use crate::wrap::Rnode;

pub struct traveller<'a>{
    stack: Vec<&'a Rnode>,
    curr: &'a Rnode
}

impl<'a> traveller<'a>{
    pub fn new(node: &'a Rnode) -> traveller{
        traveller { stack: vec![node], curr: node }
    }
}

impl<'a> Iterator for traveller<'a>{
    type Item = &'a Rnode;
    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() == 0{
            return None;
        }
        let tmp = self.stack.pop().unwrap();
        self.curr = tmp;
        for child in (0..self.curr.children_with_tokens.len()).rev(){
            self.stack.push(
                &self.curr.children_with_tokens[child]
            );
        }
        Some(tmp)
    }
}