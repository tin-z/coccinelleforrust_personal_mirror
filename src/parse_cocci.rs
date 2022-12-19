use crate::make_parsable::PREFIX;

pub fn parse(contents: &str){
    let plen = PREFIX.chars().count();
    for line in contents.lines(){
        match line.get(..plen){
            Some(PREFIX) => { }
            _ => { continue; }
        }

        let line: &str = &line[plen..];
        match line.chars().nth(0) {
            Some('+') => {}
            Some('-') => {}
            _ => {}
        }
    }
}