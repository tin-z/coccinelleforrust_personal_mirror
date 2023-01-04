use crate::commons;

type VirtualPosition = (commons::PositionInfo, usize);

pub enum ParseInfo {
    /* Present both in ast and list of tokens */
    OriginTok(commons::PositionInfo),
    /* Present only in ast and generated after parsing. Used mainly
    * by Julia, to add stuff at virtual places, beginning of func or decl */
    FakeTok(String, VirtualPosition),
    /* Present both in ast and list of tokens.  */
    ExpandedTok(commons::PositionInfo, VirtualPosition),
    AbstractLineTok(commons::PositionInfo)
}