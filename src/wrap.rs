use crate::visitor_ast0::work_node;
use ide_db::line_index::{LineCol, LineIndex};
use parser::SyntaxKind;
use syntax::ast::Type;
use syntax::{AstNode, SourceFile, SyntaxElement, SyntaxNode, SyntaxToken};

#[derive(PartialEq, Clone)]
pub struct Rnode {
    pub wrapper: wrap,
    pub astnode: SyntaxElement,
    pub children: Vec<Rnode>,
    pub children_with_tokens: Vec<Rnode>
}

impl Rnode {
    pub fn new_root(wrapper: wrap, syntax: SyntaxElement, children: Vec<Rnode>, 
        children_with_tokens: Vec<Rnode>) -> Rnode {
        Rnode {
            wrapper: wrapper,
            astnode: syntax,
            children: children,
            children_with_tokens: children_with_tokens
        }
    }

    pub fn set_children(&mut self, children: Vec<Rnode>) {
        self.children = children
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
}

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
    pub fn new(
        line_start: u32,
        line_end: u32,
        logical_start: u32,
        logical_end: u32,
        column: u32,
        offset: u32,
    ) -> position_info {
        position_info {
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
pub enum mcodekind {
    //TODO
    MINUS(),
    PLUS(),
    CONTEXT(),
    MIXED(),
}

#[derive(Clone, PartialEq)]
pub struct dots_bef_aft {}

#[derive(Clone, PartialEq)]
pub struct info {
    pos_info: position_info,
    attachable_start: bool,
    attachable_end: bool,
    mcode_start: Vec<mcodekind>,
    mcode_end: Vec<mcodekind>,
    strings_before: Vec<(dummy, position_info)>,
    strings_after: Vec<(dummy, position_info)>,
    isSymbolIdent: bool,
}

impl info {
    pub fn new(
        pos_info: position_info,
        attachable_start: bool,
        attachable_end: bool,
        mcode_start: Vec<mcodekind>,
        mcode_end: Vec<mcodekind>,
        strings_before: Vec<(dummy, position_info)>,
        strings_after: Vec<(dummy, position_info)>,
        isSymbolIdent: bool,
    ) -> info {
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

#[derive(Clone, PartialEq)]
pub struct wrap {
    info: info,
    index: u32,
    pub mcodekind: mcodekind,
    exp_ty: Option<Type>,
    bef_aft: dots_bef_aft,
    true_if_meta: bool,
    true_if_arg: bool,
    pub true_if_test: bool,
    pub true_if_test_exp: bool,
    iso_info: Vec<(String, dummy)>,
}

impl wrap {
    //Since we are hashing this with Syntax eventually, do we really need the node f
    pub fn new(
        info: info,
        index: u32,
        mcodekind: mcodekind,
        exp_ty: Option<Type>,
        bef_aft: dots_bef_aft,
        true_if_meta: bool,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, dummy)>,
    ) -> wrap {
        wrap {
            info: info,
            index: index,
            mcodekind: mcodekind,
            exp_ty: exp_ty,
            bef_aft: bef_aft,
            true_if_meta: true_if_meta,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
        }
    }

    pub fn getlineno(&self) -> u32 {
        self.info.pos_info.line_start + 1
    }

    pub fn is_ident(&self) -> bool {
        self.info.isSymbolIdent
    }
}

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> wrap {
    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());
    let mut nl: usize = 0;
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
    let pos_info: position_info = position_info::new(
        sindex.line,
        eindex.line,
        sindex.line,
        eindex.line - (nl as u32),
        sindex.col,
        node.text_range().start().into(),
    );

    let info = info::new(
        pos_info,
        false,
        false,
        vec![],
        vec![],
        vec![],
        vec![],
        false,
    );
    let wrap: wrap = wrap::new(
        info,
        0,
        mcodekind::MIXED(),
        None, //will be filled later with type inference
        dots_bef_aft {},
        false,
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
    let wrap_node = &|node: SyntaxElement, df: &dyn Fn(&SyntaxElement) -> (Vec<Rnode>, Vec<Rnode>)| -> Rnode {
        let wrapped = fill_wrap(&lindex, &node);
        let (children, children_with_tokens) = df(&node);
        let rnode = Rnode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children: children,
            children_with_tokens: children_with_tokens
        };
        rnode
    };
    work_node(wrap_node, SyntaxElement::Node(root.syntax().clone()))
}
