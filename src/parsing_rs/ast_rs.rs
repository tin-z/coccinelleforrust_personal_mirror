use crate::commons::info;
use crate::parsing_cocci::ast0::Mcodekind;
type VirtualPosition = (info::PositionInfo, usize);

pub enum ParseInfo {
    /* Present both in ast and list of tokens */
    OriginTok(info::PositionInfo),
    /* Present only in ast and generated after parsing. Used mainly
    * by Julia, to add stuff at virtual places, beginning of func or decl */
    FakeTok(String, VirtualPosition),
    /* Present both in ast and list of tokens.  */
    ExpandedTok(info::PositionInfo, VirtualPosition),
    AbstractLineTok(info::PositionInfo)
}

pub enum Danger {
    DangerStart,
    DangerEnd,
    Danger,
    NoDanger
}

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