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
use std::{mem::take, ops::Deref, vec, rc::Rc};

use super::ast0::{wrap_root, MetaVar, Snode};
use crate::{commons::util, syntaxerror};
use parser::SyntaxKind;
use syntax::ast::Meta;

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

fn getrule<'a>(rules: &'a Vec<Rule>, rulename: &str, lino: usize) -> &'a Rule<'a> {
    for rule in rules {
        if rule.name.eq(rulename) {
            return rule;
        }
    }
    syntaxerror!(lino, "no such rule", rulename);
}

/// Given a metavar type and name, returns a MetaVar object
fn getmetavarref<'a>(
    rules: &Vec<Rule<'a>>,
    rulename: &Name,
    varname: &Name,
    metatype: &str,
    lino: usize,
) -> MetaVar<'a> {
    let split = varname.split(".").collect::<Vec<&str>>();
    match (split.get(0), split.get(1), split.get(2)) {
        (Some(var), None, None) => {
            
            MetaVar::new(rulename, var, metatype)

        }
        (Some(rulen), Some(varn), None) => {
            if let Some(rule) = rules.iter().find(|x| x.name==rulen.deref()) {
                // find meta var that this references, and return  that reference 
                if let Some(mvar) = rule.metavars.iter().find(|x| {
                    return x.getname()==varn.deref() && 
                        x.getrulename()==rulen.deref() && 
                        x.gettype()==metatype;
                }) {
                    return MetaVar::Inherited(mvar);
                }
                else {
                    syntaxerror!(
                        lino,
                        format!("no such metavariable in rule {}", rulename),
                        varname
                    )
                }
            }
            else {
                syntaxerror!(
                    lino,
                    format!("No such rule {}", rulename),
                    varname
                )
            }
            
        }
        _ => syntaxerror!(lino, "Invalid meta-variable name", varname),
    }
}

pub struct Patch<'a> {
    pub minus: Snode<'a>,
    pub plus: Snode<'a>,
}

impl<'a> Patch<'a> {
    fn setmetavars_aux(node: &'a mut Snode<'a>, metavars: &'a Vec<MetaVar>) {
        let mut work = |node: &mut Snode<'a>| match node.kind() {
            Tag::PATH_EXPR => {
                let stmp = node.astnode.to_string();
                if let Some(mvar) = metavars.iter().find(|x| x.getname() == stmp) {
                    println!("MetaVar found - {:?}", mvar);
                    node.wrapper.metavar = Some(mvar);
                }
            }
            _ => {}
        };
        util::worktree(node, &mut work);
    }

    fn setmetavars(&'a mut self, metavars: &'a Vec<MetaVar>) {
        Patch::setmetavars_aux(&mut self.plus, metavars);
        Patch::setmetavars_aux(&mut self.minus, metavars);
    }
}

pub struct Rule<'a> {
    pub name: Name,
    pub dependson: Dep,
    pub metavars: Rc<&'a Vec<MetaVar<'a>>>,
    pub patch: Patch<'a>,
    pub freevars: Vec<Name>,
}

impl<'a> Rule<'a> {
    pub fn new(
        name: Name,
        dependson: Dep,
        metavars: Rc<&'a Vec<MetaVar>>,
        patch: Patch<'a>,
        freevars: Vec<Name>,
    ) -> Rule<'a> {
        let mut rule = Rule {
            name,
            dependson,
            metavars,
            patch,
            freevars,
        };
        rule
    }
}

// Given the depends clause it converts it into a Dep object
fn getdep(rules: &Vec<Rule>, lino: usize, dep: &mut Snode) -> Dep {
    let node = &dep.astnode;
    dep.print_tree();
    match node.kind() {
        Tag::PREFIX_EXPR => {
            //for NOT depends
            let [cond, expr] = util::tuple_of_2(&mut dep.children);
            match cond.kind() {
                Tag::BANG => Dep::AntiDep(Box::new(getdep(rules, lino, expr))),
                _ => syntaxerror!(lino, "Dependance must be a boolean expression"),
            }
        }
        Tag::BIN_EXPR => {
            let [lhs, cond, rhs] = util::tuple_of_3(&mut dep.children);
            match cond.kind() {
                Tag::AMP2 => {
                    //Recurses
                    Dep::AndDep(Box::new((
                        getdep(rules, lino, lhs),
                        getdep(rules, lino, rhs),
                    )))
                }
                Tag::PIPE2 => {
                    //Recurses
                    Dep::OrDep(Box::new((
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
                //IndexMap trait
                Dep::Dep(name)
            } else {
                syntaxerror!(lino, "no such Rule", name)
            }
        }
        Tag::PAREN_EXPR => {
            let expr = &mut dep.children[1];
            getdep(rules, lino, expr)
        }
        _ => syntaxerror!(lino, "malformed Rule", dep.astnode.to_string()),
    }
}

fn get_blxpr(contents: &str) -> Snode {
    wrap_root(contents)
        .children
        .swap_remove(0) //Fn
        .children
        .swap_remove(4) //BlockExpr
}

fn get_expr(contents: &str) -> Snode {
    //assumes that a
    //binary expression exists

    get_blxpr(contents) //BlockExpr
        .children
        .swap_remove(0) //StmtList
        .children
        .swap_remove(2) //TailExpr
}

/// Parses the depends on clause in the rule definition by calling getdep
fn getdependson(rules: &Vec<Rule>, rule: &str, lino: usize) -> Dep {
    let fnstr = format!("fn coccifn {{ {} }}", rule);
    getdep(rules, lino, &mut get_expr(fnstr.as_str()))
}

/// Deals with the first line of a rule definition
fn handlerules(rules: &Vec<Rule>, decl: Vec<&str>, lino: usize) -> (Name, Dep) {
    let decl = decl.join("\n");
    let mut tokens = decl.trim().split([' ', '\n']);
    let currrulename = {
        if let Some(currrulename) = tokens.next() {
            Name::from(currrulename) //converted &str to Name,
                                     //because rule should own its name
        } else {
            format!("rule{lino}")
        } //if currrulename does not exist
    };

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
fn getpatch<'a>(plusbuf: &str, minusbuf: &str, llino: usize) -> Patch<'a> {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    let mut p = Patch {
        plus: wrap_root(plusbuf.as_str()),
        minus: wrap_root(minusbuf.as_str()),
    };
    p
}

/// Given all the info about name, depends, metavars and modifiers and context
/// it consolidates everything into a line preserved rule object
fn buildrule<'a>(
    currrulename: &Name,
    currdepends: Dep,
    metavars: Rc<&'a Vec<MetaVar>>,
    blanks: usize,
    pbufmod: &String,
    mbufmod: &String,
    lastruleline: usize
) -> Rule<'a> {
    //end of the previous rule
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    plusbuf.push_str(format!("fn {currrulename}_plus() {{\n").as_str());
    minusbuf.push_str(format!("fn {currrulename}_minus() {{\n").as_str());

    plusbuf.push_str(&"\n".repeat(blanks));
    minusbuf.push_str(&"\n".repeat(blanks));

    plusbuf.push_str(pbufmod);
    minusbuf.push_str(mbufmod);

    plusbuf.push_str("}");
    minusbuf.push_str("}");

    let mut currpatch = getpatch(&plusbuf, &minusbuf, lastruleline);
    let rule = Rule::new(
        Name::from(currrulename),
        currdepends,
        metavars,
        currpatch,
        vec![]
    );
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

/// Parses the metavar declarations
pub fn handle_metavar_decl<'a>(
    rules: &'a Vec<Rule>,
    block: &Vec<&str>,
    rulename: &Name,
    lino: usize
) -> (Vec<MetaVar<'a>>, usize) {
    let mut offset: usize = 0;
    let mut blanks: usize = 0;
    let mut metavars: Vec<MetaVar> = vec![]; //stores the mvars encountered as of now

    for line in block {
        offset += 1;
        let line = line.trim();
        if line.deref() == "" {
            continue;
        }
        let mut tokens = line.split(&[',', ' ', ';'][..]);
        let ty = tokens.next().unwrap().trim();
        for var in tokens {
            let var = var.trim().to_string();
            if var != "" {
                if !metavars.iter().any(|x| x.getname() == var) {
                    metavars.push(getmetavarref(rules, rulename, &var, ty, lino));
                } else {
                    syntaxerror!(
                        offset + lino,
                        format!("Redefining {} metavariable {}", ty, var)
                    );
                }
            }
        }
        blanks += 1;
    }
    (metavars, blanks)
}

fn handleprepatch(contents: &str) {
    if contents.trim() != "" {
        syntaxerror!(0, "SyntaxError");
    }
}

pub fn processcocci(contents: &str) -> (Vec<Rule>, Vec<Vec<MetaVar>>) {
    let mut blocks: Vec<&str> = contents.split("@").collect();
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut rules: Vec<Rule> = vec![];
    let mut gmetavars: Vec<Vec<MetaVar>> = vec![];


    //check for empty
    if blocks.len() == 0 {
        return (vec![], vec![]);
    }
    //handleprepatch(blocks.swap_remove(0)); //throwing away the first part before the first @
    handleprepatch(blocks.remove(0));
    let nrules = blocks.len() / 4; //this should always be an integer if case of a proper cocci file
                                   //if it fails we will find out in the next for loop
    ;
    let mut lastruleline = 0;
    
    let (_, gmetavarsref) = gmetavars.split_at_mut(0);
    for i in 0..nrules {
        let block1: Vec<&str> = blocks[i * 4].trim().lines().collect(); //rule
        let block2: Vec<&str> = blocks[i * 4 + 1].lines().collect(); //metavars
        let block3: Vec<&str> = blocks[i * 4 + 2].lines().collect(); //empty
        let block4: Vec<&str> = blocks[i * 4 + 3].lines().collect(); //mods

        //getting rule info
        let (currrulename, currdepends) = handlerules(&rules, block1, lino);

        lino += 1;
        let (metavars, blanks) = handle_metavar_decl(&rules, &block2, &currrulename, lino);
        gmetavars.push(metavars);
        println!("lino1 - {}", lino);
        //metavars
        lino += block2.len();
        println!("lino2 - {}", lino);
        //just checks that nothing exists between the two @@
        println!("lineo3 - {:?}", block3);
        if block3.len() != 0 {
            syntaxerror!(lino, "Syntax Error");
        }

        //modifiers
        lino += block4.len() - 1;
        let (pbufmod, mbufmod) = handlemods(&block4);
        //let (prevgmeta, tmp) = gmetavars.split_at_mut(gmetavars.len()-1);
        //gmetavarsref = tmp;
        let rule = buildrule(
            &currrulename,
            currdepends,
            Rc::clone(gmetavars.last().as_ref().unwrap()),
            blanks,
            &pbufmod,
            &mbufmod,
            lastruleline
        );
        rules.push(rule);

        lastruleline = lino;
    }
    (rules, gmetavars)
    //flag_logilines(0, &mut root);
}
