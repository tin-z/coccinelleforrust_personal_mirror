#![feature(try_blocks)]

use std::{io::Lines, vec};

use parser::SyntaxKind;

use crate::{
    syntaxerror,
    util::{tuple_of_2, tuple_of_3},
    wrap::{metatype, wrap_root, Rnode},
};

type Tag = SyntaxKind;
type Name = String;

pub enum dep {
    NoDep,
    FailDep,
    Dep(Name),
    AndDep(Box<(dep, dep)>),
    OrDep(Box<(dep, dep)>),
    AntiDep(Box<dep>),
}

#[derive(PartialEq)]
pub struct mvar {
    rulename: String,
    varname: Name,
    metatype: metatype,
}

pub struct patch {
    pub minus: Rnode,
    pub plus: Rnode,
}

pub struct rule {
    pub name: Name,
    pub dependson: dep,
    pub metavars: Vec<mvar>,
    pub patch: patch,
}

fn getdep(rules: &Vec<rule>, lino: usize, dep: &mut Rnode) -> dep {
    let node = &dep.astnode;
    match node.kind() {
        Tag::PREFIX_EXPR => {
            //for NOT depends
            let [cond, expr] = tuple_of_2(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::BANG => dep::AntiDep(Box::new(getdep(rules, lino, expr))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression"),
            }
        }
        Tag::BIN_EXPR => {
            let [lhs, cond, rhs] = tuple_of_3(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::AMP2 => dep::AndDep(Box::new((
                    getdep(rules, lino, lhs),
                    getdep(rules, lino, rhs),
                ))),
                Tag::PIPE2 => dep::OrDep(Box::new((
                    getdep(rules, lino, lhs),
                    getdep(rules, lino, rhs),
                ))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression"),
            }
        }
        Tag::PATH_EXPR => {
            let name = dep.astnode.to_string();
            if rules.iter().any(|x| x.name == name) {
                //IndexMap trait
                dep::Dep(name)
            } else {
                syntaxerror!(lino, "no such Rule", name)
            }
        }
        Tag::PAREN_EXPR => {
            let expr = &mut dep.children_with_tokens[1];
            getdep(rules, lino, expr)
        }
        _ => syntaxerror!(lino, "malformed Rule", dep.astnode.to_string()),
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
    pub fn new(name: Name, patch: patch) -> rule {
        rule {
            name: name,
            dependson: dep::NoDep,
            metavars: vec![],
            patch: patch,
        }
    }
}

fn getdependson(rules: &Vec<rule>, rule: &str, lino: usize) -> dep {
    //rule is trimmed
    let fnstr = format!("fn coccifn {{ {} }}", rule);
    getdep(rules, lino, &mut get_expr(fnstr.as_str()))
}

fn tometatype(ty: &str) -> metatype {
    match ty {
        "identifier" => metatype::Id,
        "expression" => metatype::Exp,
        _ => metatype::NoMeta,
    }
}

fn handlemetavars(rulename: &Name, metavars: &mut Vec<mvar>, line: &str, lino: usize) {
    let mut tokens = line.split(&[',', ' ', ';'][..]);
    let ty = tokens.next().unwrap().trim();
    let mtype = tometatype(ty);
    if mtype != metatype::NoMeta {
        for var in tokens {
            //does not check for ; at the end of the line
            //TODO
            let var = var.trim().to_string();
            if var != "" {
                if !metavars.iter().any(|x| x.varname == var) {
                    metavars.push(mvar {
                        rulename: Name::from(rulename),
                        varname: var,
                        metatype: mtype,
                    }); //integrate metavar inheritance TODO
                } else {
                    syntaxerror!(lino, format!("Redefining {} metavariable {}", ty, var));
                }
            }
        }
    } else {
        syntaxerror!(lino, format!("No metavariable type named: {}", ty));
    }
}

fn handlerules(rules: &Vec<rule>, decl: Name, lino: usize) -> (Name, dep) {
    let mut tokens = decl.trim().split(" ");
    let currrulename = if let Some(currrulename) = tokens.next() {
        Name::from(currrulename) //converted &str to Name,
                                 //because rule should own its name
    } else {
        format!("rule{lino}")
    }; //if currrulename does not exist

    let sword = tokens.next();
    let tword = tokens.next();

    let depends = 
        match (sword, tword) {
            (Some("depends"), Some("on")) => {
                let booleanexp: Name = tokens.collect();
                getdependson(rules, Name::from(booleanexp).as_str(), lino)
            }
            (None, None) => dep::NoDep,
            _ => syntaxerror!(lino, "")     
        };

    (currrulename, depends)
}

fn getpatch(plusbuf: &str, minusbuf: &str, llino: usize) -> patch {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    patch {
        plus: wrap_root(plusbuf.as_str()),
        minus: wrap_root(minusbuf.as_str()),
    }
}

fn ismetavar(rule: &mut rule, node: &mut Rnode) -> metatype {
    let varname = node.astnode.to_string();
    for var in &rule.metavars {
        if varname.eq(&var.varname) {
            return var.metatype;
        }
    }
    metatype::NoMeta
}

fn flag_metavars(rule: &mut rule, node: &mut Rnode) {
    for mut child in node.children_with_tokens.iter_mut() {
        match (child.kind(), ismetavar(rule, child)) {
            (Tag::PATH_EXPR, a) => {
                child.wrapper.metatype = a;
            }
            _ => {
                flag_metavars(rule, child);
            }
        }
    }
}

pub fn processcocci(contents: &str) -> Vec<rule> {
    let mut blocks: Vec<&str> = contents.split("@").collect();
    let mut inmetadec = false; //checks if in metavar declaration
    let mut lino = 1; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut plusbuf = Name::from("\n");
    let mut minusbuf = Name::from("\n"); //change parsed to buf TODO

    let mut rules: Vec<rule> = vec![];
    let mut metavars: Vec<mvar> = vec![];

    let mut currrulename = Name::from("");
    let mut lastruleline = 0;
    let mut currdepends = dep::NoDep;
    blocks.remove(0);//throwing away the first part before the first @
    let nrules = blocks.len()/4; //this should always be an integer if case of a proper cocci file
    //if it fails we will find out in the next for loop

    //TODO line numbers matching properly

    for i in 0..nrules {
        let mut block = blocks[i*4];
        block = block.trim();

        //Rule declaration
        if currrulename != "" {
            //end of the previous rule
            plusbuf.push_str("}\n");
            minusbuf.push_str("}\n");

            let currpatch = getpatch(plusbuf.as_str(), minusbuf.as_str(), lastruleline);
            let rule = rule {
                name: currrulename,
                dependson: currdepends,
                metavars: metavars,
                patch: currpatch,
            };

            metavars = vec![];
            plusbuf = Name::from("");
            minusbuf = Name::from("");

            //flag_metavars(&mut rule, &mut rule.patch.plus);
            //flag_metavars(&mut rule, &mut rule.patch.minus);
            rules.push(rule);
            println!("{}", rules.last().unwrap().patch.minus.astnode.to_string());
            lastruleline = lino;
        }

        (currrulename, currdepends) = handlerules(&mut rules, String::from(block), lino);

        //actual meta vars
        block = blocks[i*4+1];
        for line in block.lines() {
            if line == "" { continue; }
            handlemetavars(&currrulename, &mut metavars, line.trim(), lino);
            plusbuf.push_str("\n"); 
            minusbuf.push_str("\n");
        }
        assert!(blocks[i*4+2] == "");
        
        //start of function
        plusbuf.push_str(format!("fn {currrulename}_plus() {{\n").as_str());
        minusbuf.push_str(format!("fn {currrulename}_minus() {{\n").as_str());

        //modifiers
        block = blocks[i*4+3];
        for line in block.lines(){
            match line.chars().next(){
                Some('+') => {
                    plusbuf.push(' ');
                    plusbuf.push_str(&line[1..]);
                    plusbuf.push('\n');
                    minusbuf.push('\n');
                },
                Some('-') => {
                    minusbuf.push(' ');
                    minusbuf.push_str(&line[1..]);
                    minusbuf.push('\n');
                    plusbuf.push('\n');
                },
                _ => {
                    plusbuf.push_str(&line[..]);
                    plusbuf.push('\n');
    
                    minusbuf.push_str(&line[..]);
                    minusbuf.push('\n');
                }
            }
        }
    }
    if currrulename != "" {
        //TODO change tofunc
        plusbuf.push('}');
        minusbuf.push('}');

        let currpatch = getpatch(plusbuf.as_str(), minusbuf.as_str(), lastruleline);
        let rule = rule {
            name: currrulename,
            dependson: currdepends,
            metavars: metavars,
            patch: currpatch,
        };
        rules.push(rule);
    }
    rules
    //flag_logilines(0, &mut root);
}
