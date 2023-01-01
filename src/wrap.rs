use crate::visitor_ast0::work_node;
use ide_db::line_index::{LineCol, LineIndex};
use parser::SyntaxKind;
use syntax::ast::Type;
use syntax::{AstNode, SourceFile, SyntaxElement, SyntaxNode, SyntaxToken};

#[derive(PartialEq, Clone)]
pub struct Rnode {
    pub wrapper: Wrap,
    pub astnode: SyntaxElement,
    pub children_with_tokens: Vec<Rnode>
}

impl Rnode {
    pub fn new_root(wrapper: Wrap, syntax: SyntaxElement, children: Vec<Rnode>, 
        children_with_tokens: Vec<Rnode>) -> Rnode {
        Rnode {
            wrapper: wrapper,
            astnode: syntax,
            children_with_tokens: children_with_tokens
        }
    }

    pub fn set_children_with_tokens(&mut self, children: Vec<Rnode>) {
        self.children_with_tokens = children
    }

    pub fn tonode(self) -> SyntaxNode{
        self.astnode.into_node().unwrap()
    }

    pub fn toktoken(self) -> SyntaxToken{
        self.astnode.into_token().unwrap()
    }

    pub fn kind(&self) -> SyntaxKind{
        self.astnode.kind()
    }

    pub fn print_tree(&self, mut pref: &mut String){//stticly debug function    
        println!("{}{:?}", pref, self.kind());
        let mut gg = pref.clone();
        gg.push_str(pref.as_str());
        for child in &self.children_with_tokens{
            child.print_tree(&mut gg)
        }
    }
    
}

#[derive(Clone, PartialEq)]
pub struct Dummy {}

#[derive(Clone, PartialEq)]
pub struct TokenInfo {
    tline_start: usize,
    tline_end: usize,
    left_offset: usize,
    right_offset: usize,
}

#[derive(Clone, PartialEq)]
pub struct PositionInfo {
    pub line_start: usize,
    pub line_end: usize,
    pub logical_start: usize,
    pub logical_end: usize,
    pub column: usize,
    pub offset: usize,
}

impl PositionInfo {
    pub fn new(
        line_start: usize,
        line_end: usize,
        logical_start: usize,
        logical_end: usize,
        column: usize,
        offset: usize,
    ) -> PositionInfo {
        PositionInfo {
            line_start: line_start,
            line_end: line_end,
            logical_start: logical_start,
            logical_end: logical_end,
            column: column,
            offset: offset,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Mcodekind {
    //TODO
    MINUS(),
    PLUS(),
    CONTEXT(),
    MIXED(),
}

#[derive(Clone, PartialEq)]
pub struct DotsBefAft {}

#[derive(Clone, PartialEq)]
pub struct Info {
    pos_info: PositionInfo,
    attachable_start: bool,
    attachable_end: bool,
    mcode_start: Vec<Mcodekind>,
    mcode_end: Vec<Mcodekind>,
    strings_before: Vec<(Dummy, PositionInfo)>,
    strings_after: Vec<(Dummy, PositionInfo)>,
    isSymbolIdent: bool,
}

impl Info {
    pub fn new(
        pos_info: PositionInfo,
        attachable_start: bool,
        attachable_end: bool,
        mcode_start: Vec<Mcodekind>,
        mcode_end: Vec<Mcodekind>,
        strings_before: Vec<(Dummy, PositionInfo)>,
        strings_after: Vec<(Dummy, PositionInfo)>,
        isSymbolIdent: bool,
    ) -> Info {
        Info {
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

#[derive(Clone, PartialEq, Copy, Debug, Hash, Eq)]
pub enum Metatype{
    NoMeta,
    Exp,
    Id
}

#[derive(Clone, PartialEq)]
pub struct Wrap {
    info: Info,
    index: usize,
    pub mcodekind: Mcodekind,
    exp_ty: Option<Type>,
    bef_aft: DotsBefAft,
    pub metatype: Metatype,
    true_if_arg: bool,
    pub true_if_test: bool,
    pub true_if_test_exp: bool,
    iso_info: Vec<(String, Dummy)>,
}

impl Wrap {
    //Since we are hashing this with Syntax eventually, do we really need the node f
    pub fn new(
        info: Info,
        index: usize,
        mcodekind: Mcodekind,
        exp_ty: Option<Type>,
        bef_aft: DotsBefAft,
        metatype: Metatype,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, Dummy)>,
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            mcodekind: mcodekind,
            exp_ty: exp_ty,
            bef_aft: bef_aft,
            metatype: metatype,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
        }
    }

    pub fn is_ident(&self) -> bool {
        self.info.isSymbolIdent
    }

    pub fn set_logilines_start(&mut self, lino: usize){
        self.info.pos_info.logical_start = lino;
    }

    pub fn set_logilines_end(&mut self, lino: usize){
        self.info.pos_info.logical_end = lino;
    }

    pub fn getlogilinenos(&self) -> (usize, usize) {
        (self.info.pos_info.logical_start,
            self.info.pos_info.logical_end)
    }
}

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> Wrap {
    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());
    let mut nl: usize = 0;
    /*
    match node {
        SyntaxElement::Node(node) => {
            for s in node.children_with_tokens() {
                s.as_token().map(|token| {
                    if token.kind() == syntax::SyntaxKind::WHITESPACE {
                        nl += token.to_string().matches('\n').count();
                    }
                });
            }
        }
        _ => {}
    }
    */ //CHECK THSI IN THE MORNING
    let pos_info: PositionInfo = PositionInfo::new(//all casted to usize because linecol returns u32
        sindex.line as usize,
        eindex.line as usize,
        0,
        0,
        sindex.col as usize,
        node.text_range().start().into(),
    );

    let info = Info::new(
        pos_info,
        false,
        false,
        vec![],
        vec![],
        vec![],
        vec![],
        false,
    );
    let wrap: Wrap = Wrap::new(
        info,
        0,
        Mcodekind::MIXED(),
        None, //will be filled later with type inference
        DotsBefAft {},
        Metatype::NoMeta,
        false,
        false,
        false,
        vec![],
    );
    wrap
}

//for wrapping
pub fn wrap_root(contents: &str) -> Rnode {
    let lindex = LineIndex::new(contents);
    let root = SourceFile::parse(contents).tree();
    let wrap_node = &|node: SyntaxElement, df: &dyn Fn(&SyntaxElement) -> Vec<Rnode> | -> Rnode {
        let wrapped = fill_wrap(&lindex, &node);
        let children_with_tokens = df(&node);
        let rnode = Rnode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children_with_tokens: children_with_tokens
        };
        rnode
    };
    work_node(wrap_node, SyntaxElement::Node(root.syntax().clone()))
}
