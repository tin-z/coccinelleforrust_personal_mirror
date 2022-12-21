pub fn make_parsable(contents: &str) -> (String, Vec<usize>){
    let mut cleaned: String = String::from("");
    let mut smpllines: Vec<usize> = vec![];
    let mut inmdec: u8 = 0;
    let mut lino = 0;
    for line in contents.lines() {
        match (line.chars().nth(0), line.chars().nth(1), inmdec) {
            (Some('+'), _, 0) | (Some('-'), _, 0) => {
                //modifiers normally
                smpllines.push(lino);
            }
            (Some('+'), _, 1) | (Some('-'), _, 1) => {
                //modifiers should not be present in @@ blocks
                panic!("Syntax Error at line {lino}");
            }
            (Some('@'), Some('@'), 0) => {
                //start of @@ block
                smpllines.push(lino);
                inmdec = 1;
            }
            (Some('@'), Some('@'), 1) => {
                //end of @@ block
                smpllines.push(lino);
                inmdec = 0;
            }
            (Some('@'), _, 1) => {
                //@@ block should always end with a @@
                panic!("Syntax Error at line {lino}");
            }
            (_, _, 1) => {
                //everything inside @@ gets commented
                smpllines.push(lino);
            }
            _ => {
                //normal lines get written as is
                cleaned.push_str(line);
            }
        }
        lino += 1;
    }

    (cleaned, smpllines)
}
