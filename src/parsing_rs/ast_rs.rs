use std::fmt::Debug;

use itertools::izip;
use parser::SyntaxKind;
use syntax::{SyntaxElement, NodeOrToken, SyntaxNode, SourceFile};
use SyntaxKind::*;

use crate::commons::info;
use crate::parsing_cocci::ast0::Mcodekind;

use super::parse_rs::fill_wrap;

type VirtualPosition = (info::ParseInfo, usize);

#[derive(Clone, PartialEq)]
pub enum ParseInfo {
    /* Present both in ast and list of tokens */
    OriginTok(info::ParseInfo),
    /* Present only in ast and generated after parsing. Used mainly
     * by Julia, to add stuff at virtual places, beginning of func or decl */
    FakeTok(String, VirtualPosition),
}

#[derive(Clone, PartialEq)]
pub enum Danger {
    DangerStart,
    DangerEnd,
    Danger,
    NoDanger,
}


#[derive(PartialEq)]
pub struct Wrap {
    pub info: ParseInfo,
    index: usize,
    cocci_tag: Option<Vec<Mcodekind>>,
    danger: Danger,
    pub wspaces: (String, bool)
}

impl Wrap {
    pub fn new(
        info: ParseInfo,
        index: usize,
        cocci_tag: Option<Vec<Mcodekind>>,
        danger: Danger,
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            cocci_tag: cocci_tag,
            danger: danger,
            wspaces: (String::new(), false)
        }
    }

    pub fn dummy() -> Wrap {
        Wrap {
            info: ParseInfo::OriginTok(info::ParseInfo::getempty()),
            index: 0,
            cocci_tag: None,
            danger: Danger::NoDanger,
            wspaces: (String::new(), false)
        }
    }
}

pub struct Rnode {
    pub wrapper: Wrap,
    pub astnode: SyntaxElement, //Not SyntaxNode because we need to take
    //care of the whitespaces
    pub children: Vec<Rnode>,
}

impl Rnode {

    pub fn headlesschildren(nodes: Vec<Rnode>) -> Rnode{
        let dummyhead = SourceFile::parse("").syntax_node();
        Rnode { wrapper: Wrap::dummy(), astnode: NodeOrToken::Node(dummyhead), children: nodes }
    }

    pub fn kind(&self) -> SyntaxKind {
        self.astnode.kind()
    }

    pub fn unwrap(&self) -> (SyntaxKind, &[Rnode]) {
        (self.kind(), &self.children[..])
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

    pub fn displaytree(&self) {
        //stticly debug function
        print!("{}", self.wrapper.wspaces.0);
        if self.children.len() == 0 {
            print!("{}", self.astnode.to_string());
        }
        else {
            for i in &self.children {
                i.displaytree();
            }
        }
    }

    pub fn isexpr(&self) -> bool {
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
            | LITERAL => true,
            _ => false,
        }
    }

    pub fn ispat(&self) -> bool {
        match self.kind() {
            IDENT_PAT | BOX_PAT | REST_PAT | LITERAL_PAT | MACRO_PAT | OR_PAT | PAREN_PAT
            | PATH_PAT | WILDCARD_PAT | RANGE_PAT | RECORD_PAT | REF_PAT | SLICE_PAT
            | TUPLE_PAT | TUPLE_STRUCT_PAT | CONST_BLOCK_PAT => true,
            _ => false,
        }
    }

    pub fn equals(&self, node: &Rnode) -> bool {
        if self.children.len() != node.children.len() {
            return false;
        } else if self.children.len() == 0 && node.children.len() == 0 {
            return self.astnode.to_string() == node.astnode.to_string();
        } else {
            for (a, b) in izip!(&self.children, &node.children) {
                if !a.equals(b) {
                    return false;
                }
            }
            return true;
        }
    }
}

impl Debug for Rnode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rnode")
            .field("astnode", &self.astnode.to_string())
            .field("children", &self.children)
            .finish()
    }
}
