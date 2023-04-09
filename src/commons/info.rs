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

#[derive(Clone)]
pub struct ParseInfo {
    str: String,
    pub charpos: usize,

    line: usize,
    column: usize,
    file: String
  }

impl ParseInfo {
    pub fn new(str: String, charpos: usize, line: usize, col: usize, file: String) -> ParseInfo {
        ParseInfo {
            str: str,
            charpos: charpos,
            line: line,
            column: col,
            file: file
        }
    }
}