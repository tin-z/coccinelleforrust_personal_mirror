/// Parse a Single .cocci file
/// Structure supported as of now
/// @ rulename (depends on bool_exp)? @
/// metavars(exp and id)
/// .
/// .
/// .
/// @@
/// 
/// _context_
/// (+/-) code

use std::{vec, ops::Deref};

use parser::SyntaxKind;

use crate::{
    syntaxerror,
    util::{tuple_of_2, tuple_of_3},
    wrap::{wrap_root, MetaVar, Rnode},
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

fn makemetavar(
    rules: &Vec<Rule>,
    rulename: &Name,
    varname: &Name,
    metatype: &str,
    lino: usize,
) -> MetaVar {
    let split = varname.split(".").collect::<Vec<&str>>();
    match (split.get(0), split.get(1), split.get(2)) {
        (Some(var), None, None) => MetaVar::new(rulename, var, metatype),
        (Some(rulen), Some(var), None) => {
            let rule = getrule(rules, &rulen, lino);
            for mvar in &rule.metavars {
                if mvar.name().eq(var.deref()) {
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

pub struct Patch {
    pub minus: Rnode,
    pub plus: Rnode,
}

impl Patch {
    fn setmetavars_aux(node: &mut Rnode, metavars: &Vec<MetaVar>) {
        match node.kind() {
            Tag::NAME_REF => {
                for metavar in metavars {
                    if node.astnode.to_string() == metavar.name() {
                        node.wrapper.metavar = metavar.clone();
                    }
                }
            }
            _ => {
                for child in &mut node.children_with_tokens {
                    Patch::setmetavars_aux(child, metavars);
                }
            }
        }
    }
    pub fn setmetavars(&mut self, metavars: &Vec<MetaVar>) {
        Patch::setmetavars_aux(&mut self.plus, metavars);
        Patch::setmetavars_aux(&mut self.minus, metavars);
    }
}

pub struct Rule {
    pub name: Name,
    pub dependson: Dep,
    pub metavars: Vec<MetaVar>,
    pub patch: Patch,
}

fn getrule<'a>(rules: &'a Vec<Rule>, rulename: &str, lino: usize) -> &'a Rule {
    for rule in rules {
        if rule.name.eq(rulename) {
            return rule;
        }
    }
    syntaxerror!(lino, "no such rule", rulename);
}

// Given the depends clause it converts it into a Dep object
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

/// Parses the depends on clause in the rule definition by calling getdep
fn getdependson(rules: &Vec<Rule>, rule: &str, lino: usize) -> Dep {
    //rule is trimmed
    let fnstr = format!("fn coccifn {{ {} }}", rule);
    getdep(rules, lino, &mut get_expr(fnstr.as_str()))
}

/// Deals with the first line of a rule definition
fn handlerules(rules: &Vec<Rule>, decl: Vec<&str>, lino: usize) -> (Name, Dep) {
    let decl = decl.get(0).unwrap();
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

/// Turns values from handlemods into a patch object
fn getpatch(plusbuf: &str, minusbuf: &str, llino: usize, metavars: &Vec<MetaVar>) -> Patch {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    let mut p = Patch {
        plus: wrap_root(plusbuf.as_str()),
        minus: wrap_root(minusbuf.as_str()),
    };
    p.setmetavars(metavars);
    p
}


/// Given all the info abuot name, depends, metavars and modifiers and context
/// it consolidates everything into a line preserved rule object
fn buildrule(
    pbufmeta: &String,
    mbufmeta: &String,
    pbufmod: &String,
    mbufmod: &String,
    currrulename: &Name,
    currdepends: Dep,
    metavars: Vec<MetaVar>,
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

    let currpatch = getpatch(&plusbuf, &minusbuf, lastruleline, &metavars);
    let rule = Rule {
        name: Name::from(currrulename),
        dependson: currdepends,
        metavars: metavars,
        patch: currpatch,
    };
    rule
}

/// Does nothing much as of now. Just appends lines inside the rules
/// while preserving line numbers with new lines
pub fn handlemods(block: &Vec<&str>) -> (String, String) {
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();

    for line in block {
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
    }
    (plusbuf, minusbuf)
}


/// Parses the metavar decalrations
pub fn handle_metavar_decl(
    rules: &Vec<Rule>,
    block: &Vec<&str>,
    currrulename: &Name,
    lino: usize,
) -> (Vec<MetaVar>, String, String) {
    let mut offset = 0;
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    let mut metavars: Vec<MetaVar> = vec![]; //stores the mvars encounteres as of now

    for line in block {
        offset += 1;
        if line.deref() == "" {
            continue;
        }
        let line = line.trim();
        let mut tokens = line.split(&[',', ' ', ';'][..]);
        let ty = tokens.next().unwrap().trim();
        for var in tokens {
            //does not check for ; at the end of the line
            //TODO
            let var = var.trim().to_string();
            if var != "" {
                if !metavars.iter().any(|x| x.name() == var) {
                    metavars.push(makemetavar(rules, currrulename, &var, ty, lino));
                //integrate metavar inheritance TODO
                } else {
                    syntaxerror!(
                        offset + lino,
                        format!("Redefining {} metavariable {}", ty, var)
                    );
                }
            }
        }
        plusbuf.push_str("//meta\n");
        minusbuf.push_str("//meta\n");
    }
    (metavars, plusbuf, minusbuf)
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
        let block1: Vec<&str> = blocks[i * 4].trim().lines().collect(); //rule
        let block2: Vec<&str> = blocks[i * 4 + 1].lines().collect(); //metavars
        let block3: Vec<&str> = blocks[i * 4 + 2].lines().collect(); //empty
        let block4: Vec<&str> = blocks[i * 4 + 3].lines().collect(); //mods

        //getting rule info
        let (currrulename, currdepends) = handlerules(&mut rules, block1, lino);

        lino += 1;
        let (metavars, pbufmeta, mbufmeta) =
            handle_metavar_decl(&mut rules, &block2, &currrulename, lino);

        
        //metavars
        lino += block2.len();
        //just checks that nothing exists between the two @@
        if block3.len()==1{
            syntaxerror!(lino, "Syntax Error");
        }

        //modifiers
        lino += block4.len()-1;
        let (pbufmod, mbufmod) = handlemods(&block4);

        //start of function
        
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
