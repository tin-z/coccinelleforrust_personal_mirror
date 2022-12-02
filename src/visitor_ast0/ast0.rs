use std::ops::Index;

use syntax::ast::{Type, MacroDef};
use syntax::SyntaxNode;

pub struct dummy{}
pub struct token_info{
    tline_start: u32, tline_end: u32,
    left_offset: u32, right_offset: u32
}
pub struct position_info{
    pub line_start: u32, pub line_end: u32,
    pub logical_start: u32, pub logical_end: u32,
    pub column: u32, pub offset: u32
}  
pub enum mcodekind<'a>{//TODO
    MINUS(&'a (dummy, token_info)),
    PLUS(),
    CONTEXT(),
    MIXED()
    
}
pub struct bef_aft{}

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

pub struct wrap<'a>{
    node: &'a SyntaxNode,
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

impl<'a> wrap<'a> {
    pub fn new(node: &'a SyntaxNode, info: info<'a>, index: u32,
    mcodekind: mcodekind<'a>, exp_ty: Option<Type>, bef_aft: bef_aft, 
    true_if_arg: bool, 
    true_if_test: bool, true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>) -> wrap<'a>{
        wrap { node: node, 
            info: info,
            index: index, mcodekind: mcodekind, 
            exp_ty: exp_ty, bef_aft: bef_aft, true_if_arg: true_if_arg, 
            true_if_test: true_if_test, true_if_test_exp: true_if_test_exp,
            iso_info: iso_info }

    }
}