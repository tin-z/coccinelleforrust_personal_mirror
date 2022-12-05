use std::cell::RefCell;
use std::ops::Index;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct Rnode<'a> {
  pub wrapper: wrap<'a>,
  pub astnode: Syntax,
  pub children: Vec<Option<Rnode<'a>>>,
}

impl<'a> Rnode<'a>{
    pub fn new_root(wrapper: wrap<'a>, syntax: Syntax, children: Vec<Option<Rnode<'a>>>) -> Rnode<'a>{
        Rnode{
            wrapper: wrapper,
            astnode: syntax,
            children: children,
        }
    }

    pub fn set_children(&mut self, children: Vec<Option<Rnode<'a>>>){
        self.children = children
    }
}

use syntax::ast::{Type, MacroDef};
use syntax::{SyntaxNode, SyntaxToken};

#[derive(Clone, PartialEq)]
pub struct dummy{}

#[derive(Clone, PartialEq)]
pub struct token_info{
    tline_start: u32, tline_end: u32,
    left_offset: u32, right_offset: u32
}

#[derive(Clone, PartialEq)]
pub struct position_info{
    pub line_start: u32, pub line_end: u32,
    pub logical_start: u32, pub logical_end: u32,
    pub column: u32, pub offset: u32
}  

#[derive(Clone, PartialEq)]
pub enum mcodekind<'a>{//TODO
    MINUS(&'a (dummy, token_info)),
    PLUS(),
    CONTEXT(),
    MIXED()
    
}

#[derive(Clone, PartialEq)]
pub struct bef_aft{}

#[derive(Clone, PartialEq)]
pub struct info<'a>{
    pos_info: position_info,
    attachable_start: bool, attachable_end: bool,
    mcode_start: Vec<mcodekind<'a>>, mcode_end: Vec<mcodekind<'a>>,
    strings_before: Vec<(dummy, position_info)>,
    strings_after: Vec<(dummy, position_info)>,
    isSymbolIdent: bool
}

impl<'a> info<'a> {
    pub fn new(pos_info: position_info, attachable_start: bool,
    attachable_end: bool, mcode_start: Vec<mcodekind<'a>>, mcode_end: Vec<mcodekind<'a>>,
    strings_before: Vec<(dummy, position_info)>,
    strings_after: Vec<(dummy, position_info)>,
    isSymbolIdent: bool ) -> info<'a>{
        info{
            pos_info: pos_info, 
            attachable_start: attachable_start, attachable_end: attachable_end,
            mcode_start: mcode_start, mcode_end: mcode_end,
            strings_before: strings_before, strings_after: strings_after,
            isSymbolIdent: isSymbolIdent
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Syntax{//TODO: Make this support attributes, visibbility, Generic Params
    Node(SyntaxNode),
    Token(SyntaxToken)
}

#[derive(Clone, PartialEq)]
pub struct wrap<'a>{
    pub syntax: Syntax,
    info: info<'a>,
    index: u32,
    mcodekind: mcodekind<'a>,
    exp_ty: Option<Type>,
    bef_aft: bef_aft,
    true_if_arg: bool,
    true_if_test: bool,
    true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>
}

impl<'a> wrap<'a> {//Since we are hashing this with Syntax eventually, do we really need the node f
    pub fn new(syntax: Syntax, info: info<'a>, index: u32,
    mcodekind: mcodekind<'a>, exp_ty: Option<Type>, bef_aft: bef_aft, 
    true_if_arg: bool, 
    true_if_test: bool, true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>) -> wrap<'a>{
        wrap { syntax: syntax, 
            info: info,
            index: index, mcodekind: mcodekind, 
            exp_ty: exp_ty, bef_aft: bef_aft, true_if_arg: true_if_arg, 
            true_if_test: true_if_test, true_if_test_exp: true_if_test_exp,
            iso_info: iso_info }
    }

    pub fn getlineno(&self) -> u32{
        self.info.pos_info.line_start
    }
}
