use std::fmt::Debug;

use crate::parsing_rs::ast_rs::Rnode;

use super::visitor_ast0::work_node;
use ide_db::line_index::{LineCol, LineIndex};
use parser::SyntaxKind;
use syntax::ast::{Type, Meta};
use syntax::{AstNode, SourceFile, SyntaxElement, SyntaxNode, SyntaxToken, NodeOrToken};

#[derive()]
/// Semantic Path Node
pub struct Snode<'a> {
    pub wrapper: Wrap<'a>,
    pub astnode: SyntaxElement,
    pub children: Vec<Snode<'a>>,
}

impl<'a> Debug for Snode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snode").field("astnode", &self.astnode.to_string()).field("children", &self.children).finish()
    }
}

impl<'a> Snode<'a> {
    pub fn new_root(
        wrapper: Wrap<'a>,
        syntax: SyntaxElement,
        children: Vec<Snode<'a>>,
    ) -> Snode<'a> {
        Snode {
            wrapper: wrapper,
            astnode: syntax,
            children: children,
        }
    }

    pub fn set_children(&mut self, children: Vec<Snode<'a>>) {
        self.children = children
    }

    pub fn tonode(self) -> SyntaxNode {
        self.astnode.into_node().unwrap()
    }

    pub fn toktoken(self) -> SyntaxToken {
        self.astnode.into_token().unwrap()
    }

    pub fn kind(&self) -> SyntaxKind {
        self.astnode.kind()
    }

    fn print_tree_aux(&self, pref: &String) {
        println!("{}{:?}", pref, self.kind());
        let mut newbuf = String::from(pref);
        newbuf.push_str(&String::from("--"));
        for child in &self.children {
            child.print_tree_aux(&newbuf)
        }
    }

    pub fn print_tree(&self) {
        //stticly debug function
        self.print_tree_aux(&String::from("--"));
    }


    pub fn isexpr(&self) -> bool {
        use SyntaxKind::{*};

        match self.kind() {
            TUPLE_EXPR
            | ARRAY_EXPR
            | PAREN_EXPR
            | PATH_EXPR
            | CLOSURE_EXPR
            | IF_EXPR
            | WHILE_EXPR
            | LOOP_EXPR
            | FOR_EXPR
            | CONTINUE_EXPR
            | BREAK_EXPR
            | BLOCK_EXPR
            | RETURN_EXPR
            | YIELD_EXPR
            | LET_EXPR
            | UNDERSCORE_EXPR
            | MACRO_EXPR
            | MATCH_EXPR
            | RECORD_EXPR
            | RECORD_EXPR_FIELD_LIST
            | RECORD_EXPR_FIELD
            | BOX_EXPR
            | CALL_EXPR
            | INDEX_EXPR
            | METHOD_CALL_EXPR
            | FIELD_EXPR
            | AWAIT_EXPR
            | TRY_EXPR
            | CAST_EXPR
            | REF_EXPR
            | PREFIX_EXPR
            | RANGE_EXPR
            | BIN_EXPR 
            | EXPR_STMT
            | LITERAL => { true }
            _ => { false }
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

#[derive(Clone, )]
pub enum Count {
    ONE,
    MINUS,
}

#[derive()]
pub enum Replacement<'a> {
    REPLACEMENT(Vec<Vec<Snode<'a>>>),
    NOREPLACEMENT,
}

#[derive()]
pub enum Befaft<'a> {
    BEFORE(Vec<Vec<Snode<'a>>>),
    AFTER(Vec<Vec<Snode<'a>>>),
    BEFOREAFTER(Vec<Vec<Snode<'a>>>, Vec<Vec<Snode<'a>>>),
    NOTHING,
}

#[derive()]
pub enum Mcodekind<'a> {
    MINUS(Replacement<'a>),
    PLUS(Count),
    CONTEXT(Befaft<'a>),
    MIXED(Befaft<'a>),
}

#[derive(Clone, PartialEq)]
pub struct DotsBefAft {}

#[derive()]
pub struct Info<'a> {
    pos_info: PositionInfo,
    attachable_start: bool,
    attachable_end: bool,
    mcode_start: Vec<Mcodekind<'a>>,
    mcode_end: Vec<Mcodekind<'a>>,
    strings_before: Vec<(Dummy, PositionInfo)>,
    strings_after: Vec<(Dummy, PositionInfo)>,
    is_symbol_ident: bool,
}

impl<'a> Info<'a> {
    pub fn new(
        pos_info: PositionInfo,
        attachable_start: bool,
        attachable_end: bool,
        mcode_start: Vec<Mcodekind<'a>>,
        mcode_end: Vec<Mcodekind<'a>>,
        strings_before: Vec<(Dummy, PositionInfo)>,
        strings_after: Vec<(Dummy, PositionInfo)>,
        is_symbol_ident: bool,
    ) -> Info<'a> {
        Info {
            pos_info: pos_info,
            attachable_start: attachable_start,
            attachable_end: attachable_end,
            mcode_start: mcode_start,
            mcode_end: mcode_end,
            strings_before: strings_before,
            strings_after: strings_after,
            is_symbol_ident,
        }
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum KeepBinding {
    UNITARY,    //Need no info
    NONUNITARY, //Need an env entry
    SAVED,      //Need a witness
}

type Minfo = (String, String, KeepBinding);//rulename, metavar name, keepbinding

#[derive(Clone, Debug)]
pub enum MetaVar<'a> {
    Exp(Minfo, Option<&'a Snode<'a>>),
    Id(Minfo, Option<&'a Snode<'a>>),
    Inherited(&'a MetaVar<'a>)
}

impl<'a> MetaVar<'a> {
    pub fn getname(&self) -> &str {
        match self {
            MetaVar::Id(minfo, _) => minfo.1.as_str(),
            MetaVar::Exp(minfo, _) => minfo.1.as_str(),
            MetaVar::Inherited(meta) => meta.getname(),
        }
    }

    pub fn gettype(&self) -> &str {
        match self {
            MetaVar::Id(_minfo, _) => "identifier",
            MetaVar::Exp(_minfo, _) => "expression",
            MetaVar::Inherited(_meta) => "inherited",
        }
    }

    pub fn setbinding(&mut self, binding: KeepBinding) {
        match self {
            Self::Exp(minfo, _) => {
                minfo.2 = binding;
            }
            Self::Id(minfo, _) => {
                minfo.2 = binding;
            }
            _ => { panic!("Cannot mutate inherited values.") }
        }
    }

    pub fn getminfo(&self) -> &Minfo {
        match self {
            Self::Exp(minfo, _) => &minfo,
            Self::Id(minfo, _) => &minfo,
            MetaVar::Inherited(meta) => meta.getminfo(),
        }
    }

    pub fn getrulename(&self) -> &str {
        match self {
            Self::Exp(minfo, _) => &minfo.0,
            Self::Id(minfo, _) => &minfo.0,
            Self::Inherited(meta) => meta.getrulename(),
        }
    }

    pub fn new(rulename: &str, name: &str, ty: &str) -> MetaVar<'a> {
        let minfo = (
            String::from(rulename),
            String::from(name),
            KeepBinding::UNITARY,
        );
        match ty {
            "expression" => MetaVar::Exp(minfo, None),
            "identifier" => MetaVar::Id(minfo, None),
            _ => panic!("Should not occur.")
        }
    }

//    pub fn makeinherited(rulename: &str, name: &str) -> MetaVar<'a> {
//        MetaVar::Inherited()
//    }

}

#[derive()]
pub struct Wrap<'a> {
    info: Info<'a>,
    index: usize,
    pub mcodekind: Mcodekind<'a>,
    exp_ty: Option<Type>,
    bef_aft: DotsBefAft,
    pub metavar: Option<&'a MetaVar<'a>>,
    true_if_arg: bool,
    pub true_if_test: bool,
    pub true_if_test_exp: bool,
    iso_info: Vec<(String, Dummy)>,
}

impl<'a> Wrap<'a> {
    pub fn new(
        info: Info<'a>,
        index: usize,
        mcodekind: Mcodekind<'a>,
        exp_ty: Option<Type>,
        bef_aft: DotsBefAft,
        metavar: Option<&'a MetaVar>,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, Dummy)>,
    ) -> Wrap<'a> {
        Wrap {
            info: info,
            index: index,
            mcodekind: mcodekind,
            exp_ty: exp_ty,
            bef_aft: bef_aft,
            metavar: metavar,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
        }
    }

    pub fn is_ident(&self) -> bool {
        self.info.is_symbol_ident
    }

    pub fn getlinenos(&self) -> (usize, usize) {
        (
            self.info.pos_info.line_start,
            self.info.pos_info.line_end
        )
    }

    pub fn set_logilines_start(&mut self, lino: usize) {
        self.info.pos_info.logical_start = lino;
    }

    pub fn set_logilines_end(&mut self, lino: usize) {
        self.info.pos_info.logical_end = lino;
    }

    pub fn getlogilinenos(&self) -> (usize, usize) {
        (
            self.info.pos_info.logical_start,
            self.info.pos_info.logical_end,
        )
    }
}

pub fn fill_wrap<'a>(lindex: &LineIndex, node: &SyntaxElement) -> Wrap<'a> {
    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());

    let pos_info: PositionInfo = PositionInfo::new(
        //all casted to usize because linecol returns u32
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
        Mcodekind::MIXED(Befaft::NOTHING),
        None, //will be filled later with type inference
        DotsBefAft {},
        None,
        false,
        false,
        false,
        vec![],
    );
    wrap
}

//for wrapping
pub fn wrap_root<'a>(contents: &str) -> Snode<'a> {
    let lindex = LineIndex::new(contents);
    let root = SourceFile::parse(contents).syntax_node();
    let wrap_node = 
        &|node: SyntaxElement, df: &dyn Fn(&SyntaxElement) -> Vec<Snode<'a>>| -> Snode<'a> {
            let wrapped = fill_wrap(&lindex, &node);
            let children = df(&node);
            let rnode = Snode {
                wrapper: wrapped,
                astnode: node, //Change this to SyntaxElement
                children: children,
            };
            rnode
    };
    work_node(wrap_node, SyntaxElement::Node(root))
}

pub enum Fixpos {
    Real(usize),
    Virt(usize, usize)
}