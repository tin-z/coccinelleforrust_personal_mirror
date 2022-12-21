use std::vec;

use syntax::{
    ast::{BlockExpr, Fn},
    AstNode, SourceFile,
};

fn get_blxpr(contents: &str) -> BlockExpr {
    let node = SourceFile::parse(contents).tree();
    let fnode = node.syntax().children().nth(0).unwrap();
    Fn::cast(fnode).unwrap().body().unwrap()
}

pub fn parse_cocci(contents: &str) {
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut inmetadec = false; //checks if in metavar declaration
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut plusstmts = String::from("");
    let mut minusstmts = String::from("");

    let mut metavars: Vec<String> = vec![];
    for line in lines {
        let mut chars = line.chars();
        let firstchar = chars.next();
        let secondchar = chars.next();
        match (firstchar, secondchar, inmetadec) {
            (Some('@'), Some('@'), false) => {
                //starting of @@ block
                let plusfn = format!("fn {}_plus {{ {} }}", "coccifn", plusstmts); //wrapping the collective statements
                let minusfn = format!("fn {}_plus {{ {} }}", "coccifn", minusstmts); //into two functions
                (get_blxpr(plusfn.as_str()), get_blxpr(minusfn.as_str())); //will work on these nodes
                inmetadec = true;
            }
            (Some('@'), Some('@'), true) => {
                //end of @@ block
                //TODO: Handle meta variables
                plusstmts = String::from("");
                minusstmts = String::from("");
                inmetadec = false;
            }
            (Some('+'), _, false) => {
                plusstmts.push_str(line.as_str());
                plusstmts.push_str("//{lino}\n");
            }
            (Some('-'), _, false) => {
                minusstmts.push_str(line.as_str());
                minusstmts.push_str("//{lino}\n");
            }
            (_, _, false) => {
                plusstmts.push_str(line.as_str());
                plusstmts.push_str("//{lino}\n");
                minusstmts.push_str(line.as_str());
                minusstmts.push_str("//{lino}\n");
            }
            (_, _, true) => {
                let mut tokens = line.split(&[',', ' ', ';'][..]);
                match tokens.next().unwrap().trim() {
                    //unwrap because there must atleast be a "" in a line
                    "expression" => {
                        for var in tokens {
                            //does not check for ; at the end of the line
                            //TODO
                            if var != "" {
                                metavars.push(var.trim().to_string());
                            }
                        }
                    }
                    "identifier" => {
                        //can expressions have the same name as identifiers?
                        //Would it not be better to create two seperate lists
                        //for ident and exp metavariables?
                        for var in tokens {
                            //does not check for ; at the end of the line
                            //TODO
                            if var != "" {
                                metavars.push(var.trim().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        lino += 1;
    }
    if inmetadec {
        panic!("Unclosed metavar declaration block at linenumber:{lino}")
    }
    //takes care of the last block
    let plusfn = format!("fn {}_plus {{ {} }}", "coccifn", plusstmts);
    let minusfn = format!("fn {}_plus {{ {} }}", "coccifn", minusstmts);
    (get_blxpr(plusfn.as_str()), get_blxpr(minusfn.as_str())); //will work on these functions
}
