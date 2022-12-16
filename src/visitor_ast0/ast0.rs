use std::vec;

use ide_db::line_index::LineIndex;
use syntax::ast::{Type};
use syntax::{SyntaxNode, SyntaxToken, AstNode};

pub struct worker<D> {//D here is a struct where we can define the data we need to track
    pub children: Vec<Vec<D>>,
    pub(self) lindex: LineIndex,
    pub(self) func_node: fn(&mut worker<D>,
                            LineIndex,
                            Box<&dyn AstNode>, 
                            &mut dyn FnMut(&mut worker<D>) -> Vec<D>) -> 
                            Option<D>,
    pub(self) func_token: fn(LineIndex, Option<SyntaxToken>) -> Option<D>
}

impl<'a, D> worker<D>{ 
    pub fn new<'b>(lindex: LineIndex, 
            f_n: fn(&mut worker<D>, LineIndex, Box<&dyn AstNode>, &mut dyn FnMut(&mut worker<D>) -> Vec<D>) -> Option<D>,
            f_t: fn(LineIndex, Option<SyntaxToken>) -> Option<D>)
        -> worker<D>{
        worker{
            children: vec![vec![]],
            lindex: lindex,
            func_node: f_n,
            func_token: f_t
        }
    }

    pub fn work_on_node(&mut self, node: Box<&dyn AstNode>, df: &mut dyn FnMut(&mut worker<D>) -> Vec<D>)
    {
        let children: Vec<D> = vec![];//this is the vecctor of children for this node
        let func = self.func_node;
        let d = func(self, self.lindex.clone(), node, df);
        let pchildren = self.children.last_mut().unwrap();//should always have one atleast
        match d {
            Some(d) => { pchildren.push(d); }//pushing this node into the children of its parents
            None => {}
        }
        self.children.push(children);//pushed away for later use

    }

    pub fn work_on_token(&mut self, token: Option<SyntaxToken>){
        let mut pchildren = self.children.last_mut().unwrap();//should always have one atleast
        let func = self.func_token;
        let d = func(self.lindex.clone(), token);
        match d {
            Some(d) => { pchildren.push(d); }//push my token
            None => {}
        }
    }

    pub fn pop_children(&mut self) -> Vec<D>{
        self.children.pop().unwrap()//pop the children of the current node and return
    }
}