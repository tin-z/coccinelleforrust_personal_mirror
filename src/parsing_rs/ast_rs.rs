// SPDX-License-Identifier: GPL-2.0

use std::fmt::Debug;
use std::fs;

use itertools::izip;
use ra_parser::SyntaxKind;
use ra_syntax::{NodeOrToken, SourceFile, SyntaxElement};
use SyntaxKind::*;

use crate::commons::info;
use crate::parsing_cocci::ast0::Mcodekind;

type VirtualPosition = (info::ParseInfo, usize);

#[derive(Clone, PartialEq)]
pub enum ParseInfo {
    /* Present both in ast and list of tokens */
    OriginTok(info::ParseInfo),
    /* Present only in ast and generated after parsing. Used mainly
     * by Julia, to add stuff at virtual places, beginning of func or decl */
    FakeTok(String, VirtualPosition),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Danger {
    DangerStart,
    DangerEnd,
    Danger,
    NoDanger,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Wrap {
    pub info: info::ParseInfo,
    index: usize,
    cocci_tag: Option<Vec<Mcodekind>>,
    danger: Danger,
    pub wspaces: (String, String),
    pub isremoved: bool,
    pub plussed: (Vec<Rnode>, Vec<Rnode>),
}

impl Wrap {
    pub fn new(
        info: info::ParseInfo,
        index: usize,
        cocci_tag: Option<Vec<Mcodekind>>,
        danger: Danger,
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            cocci_tag: cocci_tag,
            danger: danger,
            wspaces: (String::new(), String::new()),
            isremoved: false,
            plussed: (vec![], vec![]),
        }
    }

    pub fn dummy(nc: usize) -> Wrap {
        let wp = if nc == 0 {
            (String::from(" "), String::from(""))
        } else {
            (String::new(), String::new())
        };
        Wrap {
            info: info::ParseInfo::new(String::new(), 0, 0, 0, 0, 0, String::new()),
            index: 0,
            cocci_tag: None,
            danger: Danger::NoDanger,
            wspaces: wp,
            isremoved: false,
            plussed: (vec![], vec![]),
        }
    }
}

#[derive(Eq, Hash, Clone)]
pub struct Rnode {
    pub wrapper: Wrap,
    pub astnode: SyntaxElement, //Not SyntaxNode because we need to take
    //care of the whitespaces
    pub children: Vec<Rnode>,
}

impl PartialEq for Rnode {
    fn eq(&self, other: &Self) -> bool {
        other.equals(other)
    }
}

impl Rnode {
    pub fn headlesschildren(nodes: Vec<Rnode>) -> Rnode {
        let dummyhead = SourceFile::parse("").syntax_node();
        Rnode { wrapper: Wrap::dummy(0), astnode: NodeOrToken::Node(dummyhead), children: nodes }
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

    pub fn gettokenstream(&self) -> String {
        let mut data = String::new();

        if self.wrapper.wspaces.0.contains("/*COCCIVAR*/") {
            data.push_str(" ");
        }
        else {
            data.push_str(&format!("{}", self.wrapper.wspaces.0));
        }
        
        //pluses before current node
        for plusbef in &self.wrapper.plussed.0 {
            data.push_str(&plusbef.gettokenstream());
            data.push(' ');
        }
        if self.children.len() == 0 && !self.wrapper.isremoved {
            data.push_str(&format!("{}", self.astnode.to_string()));
        } else {
            for i in &self.children {
                data.push_str(&i.gettokenstream());
            }
        }
        //println!("modprogress2 - {}", data);
        //plusses after current node
        for plusaft in &self.wrapper.plussed.1 {
            //    println!("plusaft - {:?}", self.astnode.to_string());
            data.push_str(&plusaft.gettokenstream());
        }
        data.push_str(&format!("{}", self.wrapper.wspaces.1));
        //println!("returning - {}", data);
        return data;
    }

    pub fn getunformatted(&self) -> String {
        let mut data = String::new();
        data.push_str(&format!("{}", self.wrapper.wspaces.0));
        //pluses before current node
        if self.wrapper.plussed.0.len() != 0 {
            data.push_str("/*COCCIVAR*/");

            for plusbef in &self.wrapper.plussed.0 {
                data.push_str(&plusbef.getunformatted());
                data.push(' ');
            }
        }
        if self.children.len() == 0 && !self.wrapper.isremoved {
            data.push_str(&format!("{}", self.astnode.to_string()));
        } else {
            for i in &self.children {
                data.push_str(&i.getunformatted());
            }
        }
        //println!("modprogress2 - {}", data);
        //plusses after current node
        if self.wrapper.plussed.1.len() != 0 {
            data.push_str("/*COCCIVAR*/");
            for plusaft in &self.wrapper.plussed.1 {
                //    println!("plusaft - {:?}", self.astnode.to_string());
                data.push_str(&plusaft.getunformatted());
            }
        }
        data.push_str(&format!("{}", self.wrapper.wspaces.1));
        //println!("returning - {}", data);
        return data;
    }

    pub fn writetreetofile(&self, filename: &str) {
        let data = self.gettokenstream();
        fs::write(filename, data).expect("Unable to write file");
    }

    pub fn isid(&self) -> bool {
        return self.kind() == NAME || self.kind() == NAME_REF || self.ispat();
    }

    pub fn ispat(&self) -> bool {
        match self.kind() {
            IDENT_PAT | BOX_PAT | REST_PAT | LITERAL_PAT | MACRO_PAT | OR_PAT | PAREN_PAT
            | PATH_PAT | WILDCARD_PAT | RANGE_PAT | RECORD_PAT | REF_PAT | SLICE_PAT
            | TUPLE_PAT | TUPLE_STRUCT_PAT | CONST_BLOCK_PAT => true,
            _ => false,
        }
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
            | LITERAL
            | NAME_REF => true,
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

    pub fn getpos(&self) -> (usize, usize) {
        (self.wrapper.info.charstart, self.wrapper.info.charend)
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
