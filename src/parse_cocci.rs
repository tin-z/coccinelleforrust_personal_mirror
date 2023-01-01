use std::{vec};

use parser::SyntaxKind;

use crate::{
    syntaxerror,
    util::{tuple_of_2, tuple_of_3},
    wrap::{Metatype, wrap_root, Rnode},
};

type Tag = SyntaxKind;
type Name = String;

pub enum Dep {
    NoDep,
    FailDep,
    Dep(Name),
    AndDep(Box<(Dep, Dep)>),
    OrDep(Box<(Dep, Dep)>),
    AntiDep(Box<Dep>),
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct Mvar {
    rulename: Name,
    varname: Name,
    metatype: Metatype,
}

impl Mvar {
    pub fn new(
        rules: &Vec<Rule>,
        rulename: &Name,
        varname: &Name,
        metatype: Metatype,
        lino: usize,
    ) -> Mvar {
        let split = varname.split(".").collect::<Vec<&str>>();
        match (split.get(0), split.get(1), split.get(2)) {
            (Some(var), None, None) => Mvar {
                rulename: String::from(rulename),
                varname: String::from(varname),
                metatype: metatype,
            },
            (Some(rule), Some(var), None) => {
                let rule = getrule(rules, rulename, lino);
                for mvar in &rule.metavars {
                    if mvar.varname.eq(varname) {
                        return mvar.clone(); //mvars are pretty small
                    }
                }
                syntaxerror!(
                    lino,
                    format!("no such metavariable in rule {}", rule.name),
                    varname
                )
            }
            _ => {
                syntaxerror!(lino, "Invalid meta-variable name", varname);
            }
        }
    }
}

pub struct Patch {
    pub minus: Rnode,
    pub plus: Rnode,
}

pub struct Rule {
    pub name: Name,
    pub dependson: Dep,
    pub metavars: Vec<Mvar>,
    pub patch: Patch,
}

fn getrule<'a>(rules: &'a Vec<Rule>, rulename: &Name, lino: usize) -> &'a Rule {
    for rule in rules {
        if rule.name.eq(rulename) {
            return rule;
        }
    }
    syntaxerror!(lino, "no such rule", rulename);
}

fn getdep(rules: &Vec<Rule>, lino: usize, dep: &mut Rnode) -> Dep {
    let node = &dep.astnode;
    match node.kind() {
        Tag::PREFIX_EXPR => {
            //for NOT depends
            let [cond, expr] = tuple_of_2(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::BANG => Dep::AntiDep(Box::new(getdep(rules, lino, expr))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression"),
            }
        }
        Tag::BIN_EXPR => {
            let [lhs, cond, rhs] = tuple_of_3(&mut dep.children_with_tokens);
            match cond.kind() {
                Tag::AMP2 => Dep::AndDep(Box::new((
                    getdep(rules, lino, lhs),
                    getdep(rules, lino, rhs),
                ))),
                Tag::PIPE2 => Dep::OrDep(Box::new((
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
                Dep::Dep(name)
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

    get_blxpr(contents) //BlockExpr
        .children_with_tokens
        .swap_remove(0) //StmtList
        .children_with_tokens
        .swap_remove(2) //TailExpr
}

impl Rule {
    //We may need to keep a track of rules?
    pub fn new(name: Name, patch: Patch) -> Rule {
        Rule {
            name: name,
            dependson: Dep::NoDep,
            metavars: vec![],
            patch: patch,
        }
    }
}

fn getdependson(rules: &Vec<Rule>, rule: &str, lino: usize) -> Dep {
    //rule is trimmed
    let fnstr = format!("fn coccifn {{ {} }}", rule);
    getdep(rules, lino, &mut get_expr(fnstr.as_str()))
}

fn tometatype(ty: &str) -> Metatype {
    match ty {
        "identifier" => Metatype::Id,
        "expression" => Metatype::Exp,
        _ => Metatype::NoMeta,
    }
}

fn handlerules(rules: &Vec<Rule>, decl: Name, lino: usize) -> (Name, Dep) {
    let mut tokens = decl.trim().split(" ");
    let currrulename = if let Some(currrulename) = tokens.next() {
        Name::from(currrulename) //converted &str to Name,
                                 //because rule should own its name
    } else {
        format!("rule{lino}")
    }; //if currrulename does not exist

    let sword = tokens.next();
    let tword = tokens.next();

    let depends = match (sword, tword) {
        (Some("depends"), Some("on")) => {
            let booleanexp: Name = tokens.collect();
            getdependson(rules, Name::from(booleanexp).as_str(), lino)
        }
        (None, None) => Dep::NoDep,
        _ => syntaxerror!(lino, ""),
    };

    (currrulename, depends)
}

fn getpatch(plusbuf: &str, minusbuf: &str, llino: usize) -> Patch {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    Patch {
        plus: wrap_root(plusbuf.as_str()),
        minus: wrap_root(minusbuf.as_str()),
    }
}

fn buildrule(
    pbufmeta: &String,
    mbufmeta: &String,
    pbufmod: &String,
    mbufmod: &String,
    currrulename: &Name,
    currdepends: Dep,
    metavars: Vec<Mvar>,
    lastruleline: usize,
) -> Rule {
    //end of the previous rule
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    plusbuf.push_str(format!("fn {currrulename}_plus() {{\n").as_str());
    minusbuf.push_str(format!("fn {currrulename}_minus() {{\n").as_str());

    plusbuf.push_str(pbufmeta);
    minusbuf.push_str(mbufmeta);

    plusbuf.push_str(pbufmod);
    minusbuf.push_str(mbufmod);

    plusbuf.push_str("}");
    minusbuf.push_str("}");

    let currpatch = getpatch(&plusbuf, &minusbuf, lastruleline);
    let rule = Rule {
        name: Name::from(currrulename),
        dependson: currdepends,
        metavars: metavars,
        patch: currpatch,
    };
    rule
}

pub fn handlemods(block: &str) -> (String, String, usize) {
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    let mut lino = 0;
    
    for line in block.lines() {
        match line.chars().next() {
            Some('+') => {
                plusbuf.push(' ');
                plusbuf.push_str(&line[1..]);
                plusbuf.push('\n');
                minusbuf.push('\n');
            }
            Some('-') => {
                minusbuf.push(' ');
                minusbuf.push_str(&line[1..]);
                minusbuf.push('\n');
                plusbuf.push('\n');
            }
            _ => {
                plusbuf.push_str(&line[..]);
                plusbuf.push('\n');

                minusbuf.push_str(&line[..]);
                minusbuf.push('\n');
            }
        }
        lino += 1;
    }
    (plusbuf, minusbuf, lino)
}


pub fn handle_metavar_decl(
    rules: &Vec<Rule>,
    block: &str,
    currrulename: &Name,
    offset: usize,
) -> (Vec<Mvar>, String, String, usize) {
    let mut lino = 0;
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    let mut metavars = vec![];
    for line in block.lines() {
        if line == "" {
            continue;
        }
        let line = line.trim();
        let mut tokens = line.split(&[',', ' ', ';'][..]);
        let ty = tokens.next().unwrap().trim();
        let mtype = tometatype(ty);
        if mtype != Metatype::NoMeta {
            for var in tokens {
                //does not check for ; at the end of the line
                //TODO
                let var = var.trim().to_string();
                if var != "" {
                    if !metavars.iter().any(|x: &Mvar| x.varname == var) {
                        metavars.push(Mvar::new(&rules, currrulename, &var, mtype, lino));
                    //integrate metavar inheritance TODO
                    } else {
                        syntaxerror!(offset+lino, format!("Redefining {} metavariable {}", ty, var));
                    }
                }
            }
        } else {
            syntaxerror!(lino, format!("No metavariable type named: {}", ty));
        }
        plusbuf.push_str("//meta\n");
        minusbuf.push_str("//meta\n");
        lino += 1;
    }
    (metavars, plusbuf, minusbuf, lino)
}


pub fn processcocci(contents: &str) -> Vec<Rule> {
    let mut blocks: Vec<&str> = contents.split("@").collect();
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut rules: Vec<Rule> = vec![];
    blocks.remove(0); //throwing away the first part before the first @
    let nrules = blocks.len() / 4; //this should always be an integer if case of a proper cocci file
                                   //if it fails we will find out in the next for loop

    //TODO line numbers matching properly
    let mut lastruleline = 0;
    for i in 0..nrules {
        let block1 = blocks[i*4].trim();//rule
        let block2 = blocks[i*4 + 1];//metavars
        let block3 = blocks[i*4 + 2];//empty
        let block4 = blocks[i*4 + 3];//mods

        //getting rule info
        let (currrulename, currdepends) =
            handlerules(&mut rules, String::from(block1), lino);
        
        let (metavars, pbufmeta, mbufmeta, ltmpmeta) =
            handle_metavar_decl(&mut rules, block2, &currrulename, lino);

        //just checks that nothing exists between the two @@
        if !(block3 == "") {
            syntaxerror!(lino, "Syntax Error");
        }

        //modifiers
        let (pbufmod, mbufmod, ltmpmod) =
            handlemods(block4);

        //start of function
        lino += 1;
        //metavars
        lino += ltmpmeta;
        //modifiers
        lino += ltmpmod;
        println!("{},{},{}", currrulename, ltmpmeta, ltmpmod);

        let rule = buildrule(
            &pbufmeta,
            &mbufmeta,
            &pbufmod,
            &mbufmod,
            &currrulename,
            currdepends,
            metavars,
            lastruleline,
        );
        rules.push(rule);

        lastruleline = lino;
    }
    rules
    //flag_logilines(0, &mut root);
}
