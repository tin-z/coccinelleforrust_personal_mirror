#![feature(try_blocks)]

use std::{vec, io::Lines};

use parser::SyntaxKind;

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
    pub fn new(rule: &str, var: &str) -> mvar {
        mvar {
            rulename: var.to_string(),
            varname: var.to_string(),
        }
    }
}

struct rule {
    name: String,
    dependson: dep,
}

fn getdep(rules: &Vec<rule>, lino: usize, dep: &mut Rnode) -> dep {
    let node = &dep.astnode;
    match node.kind() {
        Tag::PREFIX_EXPR => {
            //for NOT depends
            let [cond, expr] = tuple_of_2(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::BANG => dep::AntiDep(Box::new(getdep(rules, lino, expr))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression")
            }
        }
        Tag::BIN_EXPR => {
            let [lhs, cond, rhs] = tuple_of_3(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::AMP2 => {
                    dep::AndDep(Box::new((
                        getdep(rules, lino, lhs),
                        getdep(rules, lino, rhs),
                    )))
                }
                Tag::PIPE2 => {
                    dep::OrDep(Box::new((
                        getdep(rules, lino, lhs),
                        getdep(rules, lino, rhs),
                    )))
                }
                _ => syntaxerror!(lino, "Dependance must be a boolean expression"),
            }
        }
        Tag::PATH_EXPR => {
            let name = dep.astnode.to_string();
            if rules.iter().any(|x| x.name == name) {
                dep::Dep(name)
            } else {
                syntaxerror!(lino, "no such Rule", name)
            }
        }
        Tag::PAREN_EXPR => {
            let expr = &mut dep.children_with_tokens[1];
            getdep(rules, lino, expr)
        }
        _ => syntaxerror!(lino, "Malformed Rule", dep.astnode.to_string())
    }
}

fn get_blxpr(contents: &str) -> Rnode {
    wrap_root(contents)
        .children_with_tokens
        .swap_remove(0) //Fn
        .children_with_tokens
        .swap_remove(4) //BlockExpr
}

fn get_expr(contents: &str) -> Rnode {
    //assumes that a
    //binary expression exists
    println!("contents - {contents}");

    get_blxpr(contents) //BlockExpr
        .children_with_tokens
        .swap_remove(0) //StmtList
        .children_with_tokens
        .swap_remove(2) //TailExpr
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

fn handlemetavars(
    rulename: &str,
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
                let var = var.trim();
                if var != "" {
                    exmetavars.push(mvar::new(rulename, var));
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
                let var = var.trim();
                if var != "" {
                    idmetavars.push(mvar::new(rulename, var));
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
        _ => syntaxerror!(lino, "")
    }

    let name = String::from(String::from(&currrule.name));
    rules.push(currrule);

    name
}

fn get_logilines(mut lino: usize, node: &mut Rnode){
    println!("into - {:?}", node.kind());
    if node.kind() == Tag::LITERAL{
        return;   
    }
    for child in &mut node.children_with_tokens{
        let mut end = 0;
        let text = child.astnode.to_string();
        if text.matches('\n').count() == 0{
            child.wrapper.set_logilines(lino, lino);
            continue;
        }

        let lines= text.lines();
        for line in lines{
            if line.trim().len() != 0{
                end+=1;
            }
        }
        if end != 0 {
            end-=1;
        }
        if child.kind() == Tag::WHITESPACE{
            end = 1;
        }
        child.wrapper.set_logilines(lino, lino + end);
        
        get_logilines(lino, child);
        println!("s={:?} -> {}------\n({}, {})", child.kind(), child.astnode.to_string(), lino, lino + end);
        lino+=end;
    }

}

pub fn processcocci(contents: &str) {
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    let mut inmetadec = false; //checks if in metavar declaration
    let mut lino = 1; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut plusparsed = String::from("\n");
    let mut minusparsed = String::from("\n");

    let mut rules: Vec<rule> = vec![]; //keeps a track of rules
    let mut idmetavars: Vec<mvar> = vec![];
    let mut exmetavars: Vec<mvar> = vec![];

    let mut rulename = String::from("");
    for line in lines {
        let chars: Vec<char> = line.trim().chars().collect();
        let firstchar = chars.get(0);
        let lastchar = chars.last();
        match (firstchar, lastchar, inmetadec) {
            (Some('@'), Some('@'), false) => {
                //starting of @@ block
                //iter and collect converts from [char] to String
                if rulename != "" {
                    plusparsed.push_str("}\n");
                    minusparsed.push_str("}\n");
                }

                rulename = handlerules(&mut rules, chars, lino);
                //(get_blxpr(plusfn.as_str()), get_blxpr(minusfn.as_str()));
                inmetadec = true;
            }
            (Some('@'), Some('@'), true) => {
                plusparsed.push_str(format!("fn {rulename}_plus() {{\n").as_str());
                minusparsed.push_str(format!("fn {rulename}_minus() {{\n").as_str());
                inmetadec = false;
            }
            (Some('+'), _, false) => {
                plusparsed.push_str(line.as_str());
                plusparsed.push('\n');
                minusparsed.push('\n');
            }
            (Some('-'), _, false) => {
                minusparsed.push_str(line.as_str());
                minusparsed.push('\n');
                plusparsed.push('\n');
            }
            (_, _, false) => {
                plusparsed.push_str(line.as_str());
                plusparsed.push('\n');

                minusparsed.push_str(line.as_str());
                minusparsed.push('\n');
            }
            (_, _, true) => {
                handlemetavars(&rulename, &mut idmetavars, &mut exmetavars, line);
                plusparsed.push('\n');
                minusparsed.push('\n')
            }
        }
        lino += 1;
    }
    if inmetadec {
        syntaxerror!(lino, "Unclosed metavariable declaration block")
    }
    if rulename != "" {
        plusparsed.push('}');
        minusparsed.push('}');
    }

    println!("{minusparsed}");
    let mut root = wrap_root(
        &minusparsed
    );

    get_logilines(0, &mut root);
    let gg = &root.children_with_tokens[0];
    println!("{{{}, {}}}", gg.wrapper.getlinenos().0, gg.wrapper.getlinenos().1);
    
    
}
