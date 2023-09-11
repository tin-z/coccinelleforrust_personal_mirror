// SPDX-License-Identifier: GPL-2.0

use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::process::exit;

use crate::commons::util::collecttree;

use super::visitor_ast0::work_node;
use itertools::Itertools;
use ra_ide_db::line_index::{LineCol, LineIndex};
use ra_parser::SyntaxKind;
use ra_syntax::ast::Type;
use ra_syntax::{SourceFile, SyntaxElement, SyntaxNode};

#[derive(PartialEq, Clone)]
/// Semantic Path Node
pub struct Snode {
    pub wrapper: Wrap,
    pub asttoken: Option<SyntaxElement>,
    kind: SyntaxKind,
    pub children: Vec<Snode>,
}

impl Hash for Snode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.wrapper.info.pos_info.cstart.hash(state);
        self.wrapper.info.pos_info.cend.hash(state);
    }
}

pub type Pluses = (Vec<Snode>, Vec<Snode>);

impl Debug for Snode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.children.len() == 0 {
            f.debug_struct("Snode")
                .field("asttoken", &self.asttoken.as_ref().unwrap().to_string())
                .finish()
        } else {
            f.debug_struct("Snode")
                .field("kind", &self.kind())
                .field("children", &self.children)
                .finish()
        }
    }
}

impl<'a> Snode {
    //pub fn new_rloot(wrapper: Wrap, syntax: SyntaxElement, children: Vec<Snode>) -> Snode {
    //    Snode { wrapper: wrapper, astnode: Some(syntax), kind: , children: children }
    //}

    pub fn set_children(&mut self, children: Vec<Snode>) {
        self.children = children
    }

    pub fn tonode(self) -> SyntaxNode {
        self.asttoken.unwrap().into_node().unwrap()
    }

    pub fn totoken(&self) -> String {
        //panics is element is node
        self.asttoken.as_ref().unwrap().to_string()
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    pub fn getstring(&self) -> String {
        if self.children.len() == 0 {
            if self.asttoken.is_none() {
                return String::new();
            }
            return self.totoken();
        } else {
            let mut tokens: String = String::new();
            for i in &self.children {
                tokens = format!("{} {}", tokens, i.getstring());
            }
            return String::from(tokens.trim());
        }
    }

    fn print_tree_aux(&self, pref: &String) {
        println!(
            "{}{:?}, {:?}: {:?}",
            pref,
            self.kind(),
            self.wrapper.mcodekind,
            self.wrapper.metavar
        );
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

    pub fn istype(&self) -> bool {
        use SyntaxKind::*;

        match self.kind() {
            ARRAY_TYPE | DYN_TRAIT_TYPE | FN_PTR_TYPE | FOR_TYPE | IMPL_TRAIT_TYPE | INFER_TYPE
            | MACRO_TYPE | NEVER_TYPE | PAREN_TYPE | PATH_TYPE | PTR_TYPE | REF_TYPE
            | SLICE_TYPE | TUPLE_TYPE => true,
            _ => false,
        }
    }

    pub fn isid(&self) -> bool {
        use SyntaxKind::*;
        return self.kind() == NAME || self.kind() == NAME_REF || self.ispat();
    }

    pub fn ispat(&self) -> bool {
        use SyntaxKind::*;
        match self.kind() {
            IDENT_PAT | BOX_PAT | REST_PAT | LITERAL_PAT | MACRO_PAT | OR_PAT | PAREN_PAT
            | PATH_PAT | WILDCARD_PAT | RANGE_PAT | RECORD_PAT | REF_PAT | SLICE_PAT
            | TUPLE_PAT | TUPLE_STRUCT_PAT | CONST_BLOCK_PAT => true,
            _ => false,
        }
    }

    pub fn isexpr(&self) -> bool {
        use SyntaxKind::*;

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
            | LITERAL
            | NAME_REF => true,
            _ => false,
        }
    }

    pub fn getdisjs(&'a self) -> (Vec<&'a Snode>, Pluses) {
        if !self.wrapper.isdisj {
            return (vec![], (vec![], vec![]));
        }
        fn collectdisjs<'b>(node: &'b Snode) -> Vec<&'b Snode> {
            //this function also returns the plus at the end of a disjunction
            let mut disjs: Vec<&Snode> = vec![];
            if node.wrapper.isdisj {
                disjs.push(&node.children[2].children[0]); //stmtlist is pushed
                match &node.children[..] {
                    [_ifkw, _cond, _block, _elsekw, ifblock] => {
                        disjs.append(&mut collectdisjs(ifblock));
                    }
                    _ => {}
                }
            }
            return disjs;
        }
        let disjs = collectdisjs(&self);
        return (disjs, self.wrapper.mcodekind.getpluses());
    }

    pub fn get_constants(&self) -> Vec<String> {
        let mut constants: HashSet<String> = HashSet::new();

        let mut f = |node: &Snode| {
            if node.kind().is_keyword() {
                constants.insert(node.totoken());
            }
        };

        collecttree(self, &mut f);

        constants.into_iter().collect_vec()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MetavarName {
    pub rulename: String,
    pub varname: String,
}

#[derive(Clone, PartialEq)]
pub struct Dummy {}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MODKIND {
    PLUS,
    MINUS,
    STAR,
}

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
    pub cstart: usize,
    pub cend: usize,
    pub offset: usize,
}

impl PositionInfo {
    pub fn new(
        line_start: usize,
        line_end: usize,
        logical_start: usize,
        logical_end: usize,
        column: usize,
        cstart: usize,
        cend: usize,
        offset: usize,
    ) -> PositionInfo {
        PositionInfo {
            line_start: line_start,
            line_end: line_end,
            logical_start: logical_start,
            logical_end: logical_end,
            column: column,
            cstart: cstart,
            cend: cend,
            offset: offset,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Info {
    pos_info: PositionInfo,
    attachable_start: bool,
    attachable_end: bool,
    strings_before: Vec<(Dummy, PositionInfo)>,
    strings_after: Vec<(Dummy, PositionInfo)>,
    is_symbol_ident: bool,
}

impl Info {
    pub fn new(
        pos_info: PositionInfo,
        attachable_start: bool,
        attachable_end: bool,
        strings_before: Vec<(Dummy, PositionInfo)>,
        strings_after: Vec<(Dummy, PositionInfo)>,
        is_symbol_ident: bool,
    ) -> Info {
        Info {
            pos_info: pos_info,
            attachable_start: attachable_start,
            attachable_end: attachable_end,
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

type Minfo = (MetavarName, KeepBinding, bool); //rulename, metavar name, keepbinding, is_inherited

#[derive(Clone, Hash, Debug, PartialEq)]
pub enum Mcodekind {
    Minus(Vec<Snode>),
    Plus,
    Context(Vec<Snode>, Vec<Snode>),
    Star,
}

impl<'a> Mcodekind {
    pub fn is_context(&self) -> bool {
        match self {
            Mcodekind::Context(_, _) => true,
            _ => false,
        }
    }

    pub fn get_plusesbef_ref_mut(&'a mut self) -> &'a mut Vec<Snode> {
        match self {
            Mcodekind::Context(a, _) => a,
            Mcodekind::Minus(a) => a,
            _ => panic!("No pluses should be attached to Plus or star nodes"),
        }
    }

    pub fn get_plusesaft_ref_mut(&'a mut self) -> &'a mut Vec<Snode> {
        match self {
            Mcodekind::Context(_, a) => a,
            Mcodekind::Minus(a) => a,
            _ => panic!("No pluses should be attached to Plus or star nodes"),
        }
    }

    pub fn get_plusesbef_ref(&'a self) -> &'a Vec<Snode> {
        match self {
            Mcodekind::Context(a, _) => &a,
            Mcodekind::Minus(a) => &a,
            _ => panic!("No pluses should be attached to Plus or star nodes"),
        }
    }

    pub fn get_plusesaft_ref(&'a self) -> &'a Vec<Snode> {
        match self {
            Mcodekind::Context(_, a) => &a,
            Mcodekind::Minus(a) => &a,
            _ => panic!("No pluses should be attached to Plus or star nodes"),
        }
    }

    //Warning: Clones plussed nodes
    pub fn getpluses(&self) -> (Vec<Snode>, Vec<Snode>) {
        match self {
            Mcodekind::Context(a, b) => (a.clone(), b.clone()),
            Mcodekind::Minus(a) => (a.clone(), vec![]),
            _ => panic!("No pluses should be attached to Plus or star nodes"),
        }
    }

    pub fn push_pluses_front(&mut self, pluses: Vec<Snode>) {
        match self {
            Mcodekind::Context(a, _) => {
                a.extend(pluses);
            }
            Mcodekind::Minus(a) => a.extend(pluses),
            _ => {
                panic!("Cannot attach plus to Plus or Star nodes");
            }
        }
    }

    pub fn push_pluses_back(&mut self, pluses: Vec<Snode>) {
        match self {
            Mcodekind::Context(_, a) => {
                a.extend(pluses);
            }
            Mcodekind::Minus(a) => a.extend(pluses),
            _ => {
                panic!("Cannot attach plus to Plus or Star nodes");
            }
        }
    }
}

#[derive(Clone, Hash, Debug, Eq)]
pub enum MetaVar {
    NoMeta,
    Exp(Minfo),
    Id(Minfo),
    Type(Minfo),
    Struct(String, Minfo), //typename, minfo
    Enum(String, Minfo),   //typename, minfo
}

impl MetaVar {
    pub fn getname(&self) -> &str {
        match self {
            Self::NoMeta => {
                panic!("Should never happen");
            }
            Self::Id(minfo) => minfo.0.varname.as_str(),
            Self::Exp(minfo) => minfo.0.varname.as_str(),
            Self::Type(minfo) => minfo.0.varname.as_str(),
            Self::Struct(_, minfo) => minfo.0.varname.as_str(),
            Self::Enum(_, minfo) => minfo.0.varname.as_str(),
        }
    }

    pub fn gettype(&self) -> &str {
        match self {
            Self::NoMeta => "None",
            Self::Id(_minfo) => "identifier",
            Self::Exp(_minfo) => "expression",
            Self::Type(_minfo) => "type",
            Self::Struct(_, _minfo) => "struct",
            Self::Enum(_, _minfo) => "enum",
        }
    }

    pub fn setbinding(&mut self, binding: KeepBinding) {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => {
                minfo.1 = binding;
            }
            Self::Id(minfo) => {
                minfo.1 = binding;
            }
            Self::Type(minfo) => {
                minfo.1 = binding;
            }
            Self::Struct(_, minfo) => {
                minfo.1 = binding;
            }
            Self::Enum(_, minfo) => {
                minfo.1 = binding;
            }
        }
    }

    pub fn getminfo(&self) -> &Minfo {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => &minfo,
            Self::Id(minfo) => &minfo,
            Self::Type(minfo) => &minfo,
            Self::Struct(_, minfo) => &minfo,
            Self::Enum(_, minfo) => &minfo,
        }
    }

    pub fn getrulename(&self) -> &str {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => &minfo.0.rulename.as_str(),
            Self::Id(minfo) => &minfo.0.rulename.as_str(),
            Self::Type(minfo) => &minfo.0.rulename.as_str(),
            Self::Struct(_, minfo) => &minfo.0.rulename.as_str(),
            Self::Enum(_, minfo) => &minfo.0.rulename.as_str(),
        }
    }

    pub fn new(rulename: &str, name: &str, ty: &MetavarType, isinherited: bool) -> Option<MetaVar> {
        
        let minfo = (
            MetavarName { rulename: rulename.to_string(), varname: name.to_string() },
            KeepBinding::UNITARY,
            isinherited,
        );
        match ty {
            MetavarType::Expression => Some(Self::Exp(minfo)),
            MetavarType::Identifier => Some(Self::Id(minfo)),
            MetavarType::Type => Some(Self::Type(minfo)),
            MetavarType::Struct(tyname) => Some(Self::Struct(tyname.clone(), minfo)),
            MetavarType::Enum(tyname) => Some(Self::Enum(tyname.clone(), minfo)),
        }
    }

    pub fn isnotmeta(&self) -> bool {
        match self {
            MetaVar::NoMeta => true,
            _ => false,
        }
    }

    pub fn makeinherited(&self) -> MetaVar{
        let mut inhertited = self.clone();
        match &mut inhertited {
            MetaVar::NoMeta => {}
            MetaVar::Exp(minfo) => {
                minfo.2 = true;
            }
            MetaVar::Id(minfo) => {
                minfo.2 = true;
            }
            MetaVar::Type(minfo) => {
                minfo.2 = true;
            }
            MetaVar::Struct(_, minfo) => {
                minfo.2 = true;
            }
            MetaVar::Enum(_, minfo) => {
                minfo.2 = true;
            }
        }

        return inhertited;
    }

    pub fn isinherited(&self) -> bool {
        match self {
            MetaVar::NoMeta => false,
            MetaVar::Exp(minfo) => minfo.2,
            MetaVar::Id(minfo) => minfo.2,
            MetaVar::Type(minfo) => minfo.2,
            MetaVar::Struct(_, minfo) => minfo.2,
            MetaVar::Enum(_, minfo) => minfo.2,
        }
    }
}

impl PartialEq for MetaVar {
    fn eq(&self, other: &Self) -> bool {
        self.getname() == other.getname() && self.getrulename() == other.getrulename()
    }
}

#[derive(Debug)]
pub enum MetavarType {
    Expression,
    Identifier,
    Type,
    Struct(String),
    Enum(String),
}

impl MetavarType {
    pub fn build(ty: &str, tyname: Option<&str>) -> MetavarType {
        match tyname {
            None => match ty {
                "expression" => MetavarType::Expression,
                "identifier" => MetavarType::Identifier,
                "type" => MetavarType::Type,
                _ => {
                    panic!("Unexpected Type")
                }
            },
            Some(tyname) => match ty {
                "struct" => MetavarType::Struct(tyname.to_string()),
                "enum" => MetavarType::Enum(tyname.to_string()),
                _ => {
                    panic!("Unexpected type.")
                }
            },
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Wrap {
    info: Info,
    index: usize,
    exp_ty: Option<Type>,
    pub metavar: MetaVar,
    true_if_arg: bool,
    pub true_if_test: bool,
    pub true_if_test_exp: bool,
    iso_info: Vec<(String, Dummy)>,
    pub isdisj: bool,
    pub mcodekind: Mcodekind,
}

impl Wrap {
    pub fn new(
        info: Info,
        index: usize,
        exp_ty: Option<Type>,
        metavar: MetaVar,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, Dummy)>,
        isdisj: bool,
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            exp_ty: exp_ty,
            metavar: metavar,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
            isdisj: isdisj,
            mcodekind: Mcodekind::Context(vec![], vec![]), //All tokens start out as context
                                                           //before being modified accordingly
        }
    }

    pub fn is_ident(&self) -> bool {
        self.info.is_symbol_ident
    }

    pub fn getlinenos(&self) -> (usize, usize) {
        (self.info.pos_info.line_start, self.info.pos_info.line_end)
    }

    pub fn set_logilines_start(&mut self, lino: usize) {
        self.info.pos_info.logical_start = lino;
    }

    pub fn set_logilines_end(&mut self, lino: usize) {
        self.info.pos_info.logical_end = lino;
    }

    pub fn getlogilinenos(&self) -> (usize, usize) {
        (self.info.pos_info.logical_start, self.info.pos_info.logical_end)
    }

    pub fn setmodkind(&mut self, modkind: String) {
        match modkind.as_str() {
            "+" => self.mcodekind = Mcodekind::Plus,
            "-" => self.mcodekind = Mcodekind::Minus(vec![]),
            "*" => self.mcodekind = Mcodekind::Star,
            _ => self.mcodekind = Mcodekind::Context(vec![], vec![]),
        }
    }
}

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> Wrap {
    let cstart = node.text_range().start();
    let cend = node.text_range().end();
    let sindex: LineCol = lindex.line_col(cstart);
    let eindex: LineCol = lindex.line_col(cend);

    let pos_info: PositionInfo = PositionInfo::new(
        //all casted to usize because linecol returns u32
        sindex.line as usize,
        eindex.line as usize,
        0,
        0,
        sindex.col as usize,
        cstart.into(),
        cend.into(),
        node.text_range().start().into(),
    );

    let info = Info::new(pos_info, false, false, vec![], vec![], false);
    let wrap: Wrap = Wrap::new(
        info,
        0,
        None, //will be filled later with type inference
        MetaVar::NoMeta,
        false,
        false,
        false,
        vec![],
        false,
    );
    wrap
}

pub fn parsedisjs<'a>(node: &mut Snode) {
    //for later
    if node.kind() == SyntaxKind::IF_EXPR {
        //println!("does it come here");
        //let ifexpr: IfExpr = IfExpr::cast(node.astnode.into_node().unwrap()).unwrap();//Just checked above
        let cond = &node.children[1]; //this gets the node for condition
        if cond.kind() == SyntaxKind::PATH_EXPR && cond.getstring() == "COCCIVAR" {
            let block = &mut node.children[2].children[0].children;
            //println!("{:?}", block[0].kind());
            block.remove(0);
            block.remove(block.len() - 1);
            node.wrapper.isdisj = true;
            //println!("december slowly creeps into my eptember heart");
        }
    }
}

//for wrapping
pub fn wrap_root(contents: &str) -> Snode {
    let lindex = LineIndex::new(contents);

    let parse = SourceFile::parse(contents);
    let errors = parse.errors();

    if errors.len() > 0 {
        for error in errors {
            let lindex = lindex.line_col(error.range().start());
            println!("Error : {} at line: {}, col {}", error.to_string(), lindex.line, lindex.col);
            println!("{}", parse.syntax_node().to_string());
            exit(1);
        }
    }

    let root = SourceFile::parse(contents).syntax_node();
    let wrap_node = &|lindex: &LineIndex,
                      node: SyntaxElement,
                      modkind: Option<String>,
                      df: &dyn Fn(&SyntaxElement) -> Vec<Snode>|
     -> Snode {
        let mut wrapped = fill_wrap(&lindex, &node);
        wrapped.setmodkind(modkind.unwrap_or(String::new()));
        let kind = node.kind();
        let children = df(&node);
        let node = if children.len() == 0 { Some(node) } else { None };
        let mut snode = Snode {
            wrapper: wrapped,
            asttoken: node, //Change this to SyntaxElement
            kind: kind,
            children: children,
        };
        parsedisjs(&mut snode);
        if snode.kind() == SyntaxKind::EXPR_STMT && snode.children.len() == 1 {
            // this means there is an expression statement without a ; at the ens
            //the reason these are removed because rust-analyzer seems to alter between
            //assigning ExprStmt and IfExprs(maybe others too)
            return snode.children.into_iter().next().unwrap();
        }
        snode
    };
    work_node(&lindex, wrap_node, SyntaxElement::Node(root), None)
}
