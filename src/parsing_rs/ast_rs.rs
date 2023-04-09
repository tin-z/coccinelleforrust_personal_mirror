use std::fmt::Debug;

use parser::SyntaxKind;
use syntax::SyntaxElement;

use crate::commons::info;
use crate::parsing_cocci::ast0::Mcodekind;
type VirtualPosition = (info::ParseInfo, usize);

#[derive(Clone)]
pub enum ParseInfo {
    /* Present both in ast and list of tokens */
    OriginTok(info::ParseInfo),
    /* Present only in ast and generated after parsing. Used mainly
    * by Julia, to add stuff at virtual places, beginning of func or decl */
    FakeTok(String, VirtualPosition)
}

#[derive(Clone)]
pub enum Danger {
    DangerStart,
    DangerEnd,
    Danger,
    NoDanger
}

#[derive(Clone)]
pub struct Wrap {
    pub info: ParseInfo,
    index: usize,
    cocci_tag: Option<Vec<Mcodekind>>,
    danger: Danger,
}

impl Wrap {
    pub fn new(
        info: ParseInfo,
        index: usize,
        cocci_tag: Option<Vec<Mcodekind>>,
        danger: Danger
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            cocci_tag: cocci_tag,
            danger: danger
        }
    }

}

#[derive(Clone)]
pub struct Rnode {
    pub wrapper: Wrap,
    pub astnode: SyntaxElement,//Not SyntaxNode because we need to take
                           //care of the whitespaces
    pub children: Vec<Rnode>
}

impl Rnode {

    pub fn kind(&self) -> SyntaxKind {
        self.astnode.kind()
    }

    pub fn unwrap(&self) -> (SyntaxKind, &[Rnode]) {
        (self.kind(), &self.children[..])
    }
}

impl Rnode {
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
}

impl Debug for Rnode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rnode").field("astnode", &self.astnode.to_string()).field("children", &self.children).finish()
    }
}