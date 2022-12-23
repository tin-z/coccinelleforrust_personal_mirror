#![feature(try_blocks)]

use std::vec;

use parser::SyntaxKind;
use syntax::{
    ast::{BinExpr, BinaryOp, BlockExpr, Expr, Fn, LogicOp, UnaryOp},
    AstNode, SourceFile,
};

use crate::{
    syntaxerror,
    util::{tuple_of_2, tuple_of_3},
    wrap::{wrap_root, Rnode},
};

type Tag = SyntaxKind;

enum dep {
    NoDep,
    FailDep,
    Dep(String),
    AndDep(Box<(dep, dep)>),
    OrDep(Box<(dep, dep)>),
    AntiDep(Box<dep>),
}
struct mvar {
    rulename: String,
    varname: String,
}

impl mvar {
    pub fn new(rule: String, var: String) -> mvar {
        mvar {
            rulename: rule,
            varname: var,
        }
    }
}

struct rule {
    name: String,
    dependson: dep,
}

impl rule {
    //We may need to keep a track of rules?
    pub fn new(name: String) -> rule {
        rule {
            name: name,
            dependson: dep::NoDep,
        }
    }

    pub fn setdependson(&mut self, rules: &Vec<rule>, rule: &str, lino: usize) {
        //rule is trimmed
        let fnstr = format!("fn {}_plus {{ {} }}", "coccifn", rule);
        self.dependson = getdep(rules, lino, &mut get_expr(fnstr.as_str()))
    }

    
}

fn getdep(rules: &Vec<rule>, lino: usize, dep: &mut Rnode) -> dep {
    let node = &dep.astnode;
    match node.kind() {
        Tag::PREFIX_EXPR => {
            //for NOT depends
            let [cond, expr] = tuple_of_2(&mut dep.children);
            match cond.kind() {
                Tag::BANG => dep::AntiDep(Box::new(getdep(rules, lino, expr))),
                _ => syntaxerror!( lino, "Dependance must be a boolean expression" ) 
            }
        }
        Tag::BIN_EXPR => {
            let [lhs, cond, rhs] = tuple_of_3(&mut dep.children);
            match cond.kind() {
                Tag::AMP2 => dep::AndDep(Box::new((
                    getdep(rules, lino, lhs),
                    getdep(rules, lino, rhs),
                ))),
                Tag::PIPE2 => dep::OrDep(Box::new((
                    getdep(rules, lino, lhs),
                    getdep(rules, lino, rhs),
                ))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression" )
            }
        }
        Tag::PATH_EXPR => {
            let name = dep.astnode.to_string();

            if rules.iter().any(|x| x.name == name) {
                dep::Dep(name)
            } else {
                println!("{:?}", name);
                syntaxerror!(lino, "No such Rule")
            }
        }
        Tag::PAREN_EXPR => {
            let expr = &mut dep.children[1];

            getdep(rules, lino, expr)
        }
        _ => {
            syntaxerror!(lino, "Malformed Rule")
        }
    }
}

fn get_blxpr(contents: &str) -> Rnode {
    wrap_root(contents)
        .children
        .swap_remove(0) //Fn
        .children
        .swap_remove(4) //BlockExpr
}

fn get_expr(contents: &str) -> Rnode {
    //assumes that a
    //binary expression exists
    println!("contents - {contents}");

    get_blxpr(contents)
        .children
        .swap_remove(0)
        .children
        .swap_remove(2)
}

fn handlemetavars(
    rulename: &String,
    idmetavars: &mut Vec<mvar>,
    exmetavars: &mut Vec<mvar>,
    line: String,
) {
    //rule here is usize because this does not represent the
    //name of the rule but the index at which it was encountered
    let mut tokens = line.split(&[',', ' ', ';'][..]);
    match tokens.next().unwrap().trim() {
        //unwrap because there must atleast be a "" in a line
        "expression" => {
            for var in tokens {
                //does not check for ; at the end of the line
                //TODO
                if var.trim() != "" {
                    exmetavars.push(mvar::new(String::from(rulename), var.trim().to_string()));
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
                if var.trim() != "" {
                    idmetavars.push(mvar::new(String::from(rulename), var.trim().to_string()));
                }
            }
        }
        _ => {}
    }
}

fn handlerules(rules: &mut Vec<rule>, chars: Vec<char>, lino: usize) -> String {
    let decl: String = chars[1..chars.len() - 1].iter().collect();
    let mut tokens = decl.trim().split(" ");
    let rulename = if let Some(rulename) = tokens.next() {
        String::from(rulename) //converted &str to String,
                               //because rule should own its name
    } else {
        format!("rule{lino}")
    }; //if rulename does not exist
    let mut currrule = rule::new(rulename);

    let sword = tokens.next();
    let tword = tokens.next();

    match (sword, tword) {
        (Some("depends"), Some("on")) => {
            let booleanexp: String = tokens.collect();
            currrule.setdependson(rules, String::from(booleanexp).trim(), lino);
        }
        (None, None) => {}
        _ => {
            syntaxerror!(lino, "")
        }
    }

    let name = String::from(String::from(&currrule.name));
    rules.push(currrule);

    name
}

pub fn parse_cocci(contents: &str) {
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut inmetadec = false; //checks if in metavar declaration
    let mut lino = 1; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut plusstmts = String::from("");
    let mut minusstmts = String::from("");

    let mut rules: Vec<rule> = vec![]; //keeps a track of rules
    let mut idmetavars: Vec<mvar> = vec![];
    let mut exmetavars: Vec<mvar> = vec![];

    let mut rulename = String::from("");
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        let firstchar = chars.get(0);
        let lastchar = chars.last();
        match (firstchar, lastchar, inmetadec) {
            (Some('@'), Some('@'), false) => {
                //starting of @@ block
                rulename = handlerules(&mut rules, chars, lino);
                //iter and collect converts from [char] to String
                let plusfn = format!("fn {rulename}_plus {{ {plusstmts} }}"); //wrapping the collective statements
                let minusfn = format!("fn {rulename}_minus {{ {minusstmts} }}"); //into two functions
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
                plusstmts.push('\n');
            }
            (Some('-'), _, false) => {
                minusstmts.push_str(format!("/*{lino}*/").as_str());
                minusstmts.push_str(line.as_str());
                minusstmts.push('\n');
            }
            (_, _, false) => {
                if line != "" {
                    plusstmts.push_str(format!("/*{lino}*/").as_str());
                    plusstmts.push_str(line.as_str());
                    plusstmts.push('\n');

                    minusstmts.push_str(format!("/*{lino}*/").as_str());
                    minusstmts.push_str(line.as_str());
                    minusstmts.push('\n');
                }
            }
            (_, _, true) => {
                handlemetavars(&rulename, &mut idmetavars, &mut exmetavars, line);
            }
        }
        lino += 1;
    }
    if inmetadec {
        syntaxerror!(lino, "Unclosed metavariable declaration block")
    }
    //takes care of the last block
    let plusfn = format!("fn {}_plus {{ {} }}", "coccifn", plusstmts);
    let minusfn = format!("fn {}_min {{ {} }}", "coccifn", minusstmts);
    (get_blxpr(plusfn.as_str()), get_blxpr(minusfn.as_str())); //will work on these functions
}
