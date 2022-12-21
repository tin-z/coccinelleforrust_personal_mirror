use syntax::{
    ast::{BlockExpr, Fn},
    AstNode, SourceFile,
};

fn get_blxpr(contents: &str) -> BlockExpr {
    let node = SourceFile::parse(contents).tree();
    let fnode = node.syntax().children().nth(0).unwrap();
    Fn::cast(fnode).unwrap().body().unwrap()
}

struct ParseStmts{
    plusstmts: String,
    minusstmts: String
}

impl ParseStmts{

    pub fn new() -> ParseStmts{
        ParseStmts{
            plusstmts: String::from(""),
            minusstmts: String::from("")
        }
    }

    fn pushplus(&mut self, s: &str){
        self.plusstmts.push_str(s);
        self.plusstmts.push('\n');
    }

    fn pushminus(&mut self, s: &str){
        self.minusstmts.push_str(s);
        self.minusstmts.push('\n');
    }

    fn getfuncs(&self) -> (BlockExpr, BlockExpr){
        let plusfn = format!("fn {}_plus {{ {} }}", "coccifn", self.plusstmts);
        let minusfn = format!("fn {}_plus {{ {} }}", "coccifn", self.minusstmts);
        
        (get_blxpr(&plusfn[..]), get_blxpr(&minusfn[..]))
    }
}

pub fn parse_cocci(contents: &str) {
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut inmetadec = false;//checks if in metavar declaration
    let mut lino = 0;//stored line numbers
    let mut parser: ParseStmts = ParseStmts::new();//mutable because I supply it with modifier statements
    for line in lines{
        let mut chars = line.chars();
        match (chars.next(), chars.next(), inmetadec){
            (Some('@'), Some('@'), false) => {
                //starting of @@ block
                let (pfunc, mfunc) = parser.getfuncs();
                inmetadec = true;
            },
            (Some('@'), Some('@'), true) => {
                //end of @@ block
                //TODO: Handle meta variables
                parser = ParseStmts::new();
                inmetadec = false;
            },
            (Some('+'), _, false) => {
                parser.pushplus(line.as_str());
            },
            (Some('-'), _, false) => {
                parser.pushminus(line.as_str());
            },
            (Some('+'), _, true) => {
                panic!("Modifiers should not be present in metavariable declarations at line:{}", lino);
            },
            (Some('-'), _, true) => {
                panic!("Modifiers should not be present in metavariable declarations at line:{}", lino);
            },
            _ => {
                parser.pushplus(line.as_str());
                parser.pushminus(line.as_str());
            }
        }
        lino += 1;
    }
    let (pfunc, mfunc) = parser.getfuncs();//getting the last rule
}
