use syntax::ast::Type;
use syntax::SyntaxNode;

pub struct dummy{}
pub struct token_info{
    tline_start: u32, tline_end: u32,
    left_offset: u32, right_offset: u32
}
pub struct position_info{
    line_start: u32, line_end: u32,
    logical_start: u32, logical_end: u32,
    column: u32, offset: u32
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
