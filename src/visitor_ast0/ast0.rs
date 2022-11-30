use syntax::ast::Type;
use syntax::SyntaxNode;

pub struct dummy{}
pub struct token_info{
    tline_start: usize, tline_end: usize,
    left_offset: usize, right_offset: usize
}
pub struct position_info{
    line_start: usize, line_end: usize,
    logical_start: usize, logical_end: usize,
    column: usize, offset: usize
}  
pub enum mcodekind<'a>{
    MINUS(&'a (dummy, token_info)),
    PLUS(),
    CONTEXT(),
    MIXED()
    
}
pub struct bef_aft{}

pub struct Info<'a>{
    pos_info: position_info,
    attachable_start: bool, attachable_end: bool,
    mcode_start: Vec<mcodekind<'a>>, mcode_end: Vec<mcodekind<'a>>,
    strings_before: Vec<(dummy, position_info)>,
    strings_after: Vec<(dummy, position_info)>,
    isSymbolIdent: bool
}

pub struct Wrap<'a>{
    node: &'a SyntaxNode,
    info: Info<'a>,
    index: u32,
    mcodekind: mcodekind<'a>,
    exp_ty: Option<Type>,
    bef_aft: bef_aft,
    true_if_arg: bool,
    true_if_test: bool,
    true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>
}
