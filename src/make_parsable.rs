use std::fs;
use syntax::SourceFile;

pub(crate) const PREFIX: &str = "//@cocci@: ";

fn append(s: &mut String, line: &str, pre: &str) {
    s.push_str(pre);
    s.push_str(line);
    s.push_str("\n");
}

pub fn make_parsable(contents: &str) -> String{
    let mut cleaned: String = String::from("");
    let mut inmdec: u8 = 0;
    let mut lino = 0;
    for line in contents.lines() {
        match (line.chars().nth(0), line.chars().nth(1), inmdec) {
            (Some('+'), _, 0) | (Some('-'), _, 0) => {
                //modifiers normally
                append(&mut cleaned, line, PREFIX);
            }
            (Some('+'), _, 1) | (Some('-'), _, 1) => {
                //modifiers should not be present in @@ blocks
                panic!("Syntax Error at line {lino}");
            }
            (Some('@'), Some('@'), 0) => {
                append(&mut cleaned, line, PREFIX);
                inmdec = 1;
            }
            (Some('@'), Some('@'), 1) => {
                append(&mut cleaned, line, PREFIX);
                inmdec = 0;
            }
            (Some('@'), _, 1) => {
                panic!("Syntax Error at line {lino}");
            }
            (_, _, 1) => {
                append(&mut cleaned, line, PREFIX);
            }
            _ => {
                append(&mut cleaned, line, "");
            }
        }
        lino += 1;
    }

    cleaned
}
