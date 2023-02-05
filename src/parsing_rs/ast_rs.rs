use parser::SyntaxKind;
use syntax::SyntaxElement;

use crate::commons::info;
use crate::parsing_cocci::ast0::Mcodekind;
type VirtualPosition = (info::PositionInfo, usize);

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
    info: ParseInfo,
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