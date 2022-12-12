use std::cell::RefCell;
use std::ops::Index;
use std::rc::Rc;
use parser::SyntaxKind;
use either::Either;

#[derive(PartialEq)]
pub struct Rnode<'a> {
    pub wrapper: wrap<'a>,
    pub astnode: Syntax,
    pub children: Vec<Option<Rnode<'a>>>,
}

impl<'a> Rnode<'a> {
    pub fn new_root(
        wrapper: wrap<'a>,
        syntax: Syntax,
        children: Vec<Option<Rnode<'a>>>,
    ) -> Rnode<'a> {
        Rnode {
            wrapper: wrapper,
            astnode: syntax,
            children: children,
        }
    }

    pub fn set_children(&mut self, children: Vec<Option<Rnode<'a>>>) {
        self.children = children
    }
}

use ide_db::LineIndexDatabase;
use ide_db::line_index::LineIndex;
use syntax::ast::edit::AstNodeEdit;
use syntax::ast::{MacroDef, Type};
use syntax::{SyntaxNode, SyntaxToken, AstNode, AstToken};

#[derive(Clone, PartialEq)]
pub struct dummy {}

#[derive(Clone, PartialEq)]
pub struct token_info {
    tline_start: u32,
    tline_end: u32,
    left_offset: u32,
    right_offset: u32,
}

#[derive(Clone, PartialEq)]
pub struct position_info {
    pub line_start: u32,
    pub line_end: u32,
    pub logical_start: u32,
    pub logical_end: u32,
    pub column: u32,
    pub offset: u32,
}

impl position_info {
    pub fn new(line_start:u32, line_end:u32, logical_start:u32, logical_end:u32, column:u32, offset:u32) -> position_info{
        position_info{
            line_start: line_start,
            line_end: line_end,
            logical_start: logical_start,
            logical_end: logical_end,
            column: column,
            offset: offset
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum mcodekind<'a> {
    //TODO
    MINUS(&'a (dummy, token_info)),
    PLUS(),
    CONTEXT(),
    MIXED(),
}

#[derive(Clone, PartialEq)]
pub struct bef_aft {}

#[derive(Clone, PartialEq)]
pub struct info<'a> {
    pos_info: position_info,
    attachable_start: bool,
    attachable_end: bool,
    mcode_start: Vec<mcodekind<'a>>,
    mcode_end: Vec<mcodekind<'a>>,
    strings_before: Vec<(dummy, position_info)>,
    strings_after: Vec<(dummy, position_info)>,
    isSymbolIdent: bool,
}

impl<'a> info<'a> {
    pub fn new(
        pos_info: position_info,
        attachable_start: bool,
        attachable_end: bool,
        mcode_start: Vec<mcodekind<'a>>,
        mcode_end: Vec<mcodekind<'a>>,
        strings_before: Vec<(dummy, position_info)>,
        strings_after: Vec<(dummy, position_info)>,
        isSymbolIdent: bool,
    ) -> info<'a> {
        info {
            pos_info: pos_info,
            attachable_start: attachable_start,
            attachable_end: attachable_end,
            mcode_start: mcode_start,
            mcode_end: mcode_end,
            strings_before: strings_before,
            strings_after: strings_after,
            isSymbolIdent: isSymbolIdent,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Syntax {
    //TODO: Make this support attributes, visibbility, Generic Params
    Node(SyntaxNode),
    Token(SyntaxToken),
}

impl Syntax {
    pub fn to_string(&self) -> String{
        match self{
            Syntax::Node(node) => {
                node.to_string()
            }
            Syntax::Token(token) => {
                token.to_string()
            }
        }
    }

    pub fn to_node(&self) -> Option<&Syntax>{
        match self{
            Syntax::Node(node) => {
                Some(self)
            }

            Syntax::Token(_) => { None }
        }
    }

    pub fn to_token(&self) -> Option<&Syntax>{
        match self{
            Syntax::Node(_) => {
                None
            }

            Syntax::Token(token) => { 
                Some(self)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct wrap<'a> {
    info: info<'a>,
    index: u32,
    mcodekind: mcodekind<'a>,
    exp_ty: Option<Type>,
    bef_aft: bef_aft,
    true_if_arg: bool,
    true_if_test: bool,
    true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>,
}

impl<'a> wrap<'a> {
    //Since we are hashing this with Syntax eventually, do we really need the node f
    pub fn new(
        info: info<'a>,
        index: u32,
        mcodekind: mcodekind<'a>,
        exp_ty: Option<Type>,
        bef_aft: bef_aft,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, dummy)>,
    ) -> wrap<'a> {
        wrap {
            info: info,
            index: index,
            mcodekind: mcodekind,
            exp_ty: exp_ty,
            bef_aft: bef_aft,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
        }
    }

    pub fn getlineno(&self) -> u32 {
        self.info.pos_info.line_start + 1
    }

    pub fn is_ident(&self) -> bool{
        self.info.isSymbolIdent
    }

}

pub struct worker<'a, D>{//D here is a struct where we can define the data we need to track
    pub(self) children: Vec<D>,
    pub(self) lindex: &'a LineIndex,
    pub(self) func_node: fn(&'a LineIndex, Box<&dyn AstNode>, bool) -> Option<D>,
    pub(self) func_token: fn(&'a LineIndex, Option<SyntaxToken>) -> Option<D>
}

impl<'a, D> worker<'a, D> {
    pub fn new(lindex: &'a LineIndex, f_n: fn(&'a LineIndex, Box<&dyn AstNode>, bool) -> Option<D>, f_t: fn(&'a LineIndex, Option<SyntaxToken>) -> Option<D>)
    -> worker<'a, D>{
        worker{
            children: vec![],
            lindex: lindex,
            func_node: f_n,
            func_token: f_t
        }
    }

    pub fn work_on_node(&mut self, node: Box<&dyn AstNode>){
        let func = self.func_node;
        let d = func(self.lindex, node, false);//false needs to be changed
        match d {
            Some(d) => { self.children.push(d); }
            None => {}
        }
    }

    pub fn work_on_token(&mut self, token: Option<SyntaxToken>){
        let func = self.func_token;
        let d = func(self.lindex, token);
        match d {
            Some(d) => { self.children.push(d); }
            None => {}
        }
    }
}