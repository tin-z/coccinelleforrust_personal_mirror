use std::vec;

use syntax::{
    ast::{BlockExpr, Fn},
    AstNode, SourceFile,
};

use crate::util::syntaxerror;

struct rule {
    name: String,
    metavars: Vec<String>,
}

impl rule {
    pub fn new(name: String) -> rule {
        rule {
            name: name,
            metavars: vec![],
        }
    }
}

fn get_blxpr(contents: &str) -> BlockExpr {
    let node = SourceFile::parse(contents).tree();
    let fnode = node.syntax().children().nth(0).unwrap();
    Fn::cast(fnode).unwrap().body().unwrap()
}

fn handlemetavars(rule: &mut rule, line: String) {
    //rule here is usize because this does not represent the
    //name of the rule but the index at which it was encountered
    let mut tokens = line.split(&[',', ' ', ';'][..]);
    match tokens.next().unwrap().trim() {
        //unwrap because there must atleast be a "" in a line
        "expression" => {
            for var in tokens {
                //does not check for ; at the end of the line
                //TODO
                if var != "" &&
                   !rule.metavars[..].contains(&String::from(var)) {
                    rule.metavars.push(var.trim().to_string());
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
                if var != "" &&
                   !rule.metavars[..].contains(&String::from(var)) {
                    rule.metavars.push(var.trim().to_string());
                }
            }
        }
        _ => {}
    }
}

fn performinheritanceofrule(rule1: &mut rule, rule2: &rule){
    rule1.metavars = rule2.metavars.to_owned();//create a copy of meta variables
}

fn handlerules(rules: &mut Vec<rule>, chars: Vec<char>, lino: usize){

    let decl: String = chars[1..chars.len() - 1].iter().collect();
    let mut tokens = decl.trim().split(" ");
    let rulename = 
    if let Some(rulename) = tokens.next() {
            String::from(rulename)//converted &str to String,
                                  //because rule should own its name
    }
    else { format!("rule{lino}") };//if rulename does not exist
    let mut currrule = rule::new(rulename);

    let sword = tokens.next();
    let tword = tokens.next();
    let rulename = tokens.next();

    match (sword, tword, rulename){
        (Some("depends"), Some("on"), Some(rulename)) => {
            let ruleno = rules.iter_mut().position(|rule| rule.name == rulename);
            match ruleno{
                Some(ruleno) => {
                    let rule = &rules[ruleno];
                    performinheritanceofrule(&mut currrule, &rule);
                }
                None => {
                    syntaxerror(lino, format!("{rulename} not defined at ").as_str());
                }
            }
        }
        (None, None, None) => {}
        _ => {
            syntaxerror(lino, "");
        }
    }

    rules.push(currrule);

}

pub fn parse_cocci(contents: &str) {
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut inmetadec = false; //checks if in metavar declaration
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut plusstmts = String::from("");
    let mut minusstmts = String::from("");

    let mut rules: Vec<rule> = vec![];
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let firstchar = chars.get(0);
        let lastchar = chars.last();
        match (firstchar, lastchar, inmetadec) {
            (Some('@'), Some('@'), false) => {
                //starting of @@ block
                handlerules(&mut rules, chars, lino);
                //iter and collect converts from [char] to String
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
                plusstmts.push_str(format!("/*{lino}*/").as_str());
                plusstmts.push_str(line.as_str());
                minusstmts.push('\n');
            }
            (Some('-'), _, false) => {
                minusstmts.push_str(format!("/*{lino}*/").as_str());
                minusstmts.push_str(line.as_str());
                minusstmts.push('\n');
            }
            (_, _, false) => {
                if line == "" {
                    continue;
                }

                plusstmts.push_str(format!("/*{lino}*/").as_str());
                plusstmts.push_str(line.as_str());
                minusstmts.push('\n');

                minusstmts.push_str(format!("/*{lino}*/").as_str());
                minusstmts.push_str(line.as_str());
                minusstmts.push('\n');
                
            }
            (_, _, true) => {
                //should not fail because inmetadec could not have been true without
                //entering the meta dec branch
                handlemetavars(rules.last_mut().unwrap(), line);
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
