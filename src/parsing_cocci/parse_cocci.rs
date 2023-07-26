// SPDX-License-Identifier: GPL-2.0

use core::panic;
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
use std::{collections::HashSet, vec};

use super::ast0::{wrap_root, MetaVar, MetavarName, Snode, MODKIND};
use crate::{
    commons::util::{self, attachback, attachfront, collecttree, removestmtbraces, worksnode},
    debugcocci, syntaxerror,
};
use parser::SyntaxKind;

type Tag = SyntaxKind;
type Name = String;

#[derive(Debug)]
pub enum Dep {
    NoDep,
    FailDep,
    Dep(Name),
    AndDep(Box<(Dep, Dep)>),
    OrDep(Box<(Dep, Dep)>),
    AntiDep(Box<Dep>),
}

fn getrule<'a>(rules: &'a Vec<Rule>, rulename: &str, lino: usize) -> &'a Rule {
    for rule in rules {
        if rule.name.eq(rulename) {
            return rule;
        }
    }
    syntaxerror!(lino, "no such rule", rulename);
}

/// Given a metavar type and name, returns a MetaVar object
fn makemetavar(
    rules: &Vec<Rule>,
    rulename: &Name,
    varname: &Name,
    metatype: &str,
    lino: usize,
) -> MetaVar {
    let split = varname.split(".").collect::<Vec<&str>>();
    match (split.get(0), split.get(1), split.get(2)) {
        (Some(var), None, None) => {
            debugcocci!("Added Metavar {}.{}", rulename, var);
            MetaVar::new(rulename, var, metatype)
                .unwrap_or_else(|| syntaxerror!(lino, "Unexpected Metavariable type"))
        }
        (Some(rulen), Some(var), None) => {
            let var = *var;
            let rule = getrule(rules, &rulen, lino);
            if let Some(mvar) = rule.metavars.iter().find(|x| x.getname() == var) {
                if let Some(minfo) = rule.unusedmetavars.iter().find(|x| x.getname() == var) {
                    syntaxerror!(
                        lino,
                        format!(
                            "Metavariable {} is unused un rule {}",
                            minfo.getname(),
                            minfo.getrulename()
                        ),
                        varname
                    );
                }
                debugcocci!("Added Metavar {}.{}", mvar.getrulename(), mvar.getname());
                return mvar.clone();
            } else {
                syntaxerror!(lino, format!("No such metavariable in rule {}", rule.name), varname)
            }
        }
        _ => syntaxerror!(lino, "Invalid meta-variable name", varname),
    }
}

pub struct Patch {
    pub minus: Snode,
    pub plus: Snode,
}

impl Patch {
    fn setmetavars(&mut self, metavars: &Vec<MetaVar>) {
        fn setmetavars_aux(node: &mut Snode, metavars: &Vec<MetaVar>) {
            let mut work = |node: &mut Snode| {
                if node.isexpr() || node.istype() || node.isid() {
                    let stmp = node.astnode.to_string();
                    if let Some(mvar) = metavars.iter().find(|x| x.getname() == stmp) {
                        //println!("MetaVar found - {:?}", mvar);
                        node.wrapper.metavar = mvar.clone();
                    }
                }
            };
            util::worktree(node, &mut work);
        }

        setmetavars_aux(&mut self.plus, metavars);
        setmetavars_aux(&mut self.minus, metavars);
    }

    fn setmods(&mut self) {
        let mut tagmods = |node: &mut Snode,
                           (lino, modkind): (usize, Option<MODKIND>)|
         -> (usize, Option<MODKIND>) {
            let (start, end) = node.wrapper.getlinenos();

            match node.wrapper.modkind {
                Some(modkind) => {
                    if start == end {
                        //debugstart
                        if node.children.len() == 0 {
                            debugcocci!(
                                "Setting {}:{:?} to modifier:- {:?}",
                                node.astnode.to_string(),
                                node.kind(),
                                modkind
                            );
                        } //debugend

                        node.wrapper.modkind = Some(modkind);
                    } else {
                        node.wrapper.modkind = None;
                    }
                    return (start, Some(modkind));
                }
                None => {
                    if lino == 0 {
                        return (0, None);
                    } else if start == lino && start == end {
                        //debugstart
                        if node.children.len() == 0 && modkind.is_some() {
                            debugcocci!(
                                "Setting {}:{:?} to modifier:- {:?}",
                                node.astnode.to_string(),
                                node.kind(),
                                modkind.unwrap()
                            );
                        } //debugend

                        node.wrapper.modkind = modkind;
                        return (lino, modkind);
                        //everytime lino is not 0, modkind is
                        //a Some value
                    } else if start == lino && start != end {
                        //this node spills onto the next line
                        return (lino, modkind);
                    }
                    return (0, None);
                }
            }
        };

        worksnode(&mut self.plus, (0, None), &mut tagmods);
        worksnode(&mut self.minus, (0, None), &mut tagmods);
    }

    pub fn striplet(&mut self, hastype: bool) {
        if !hastype {
            return;
        }

        fn striplet_aux(node: &mut Snode) {
            //at this point node is a SourceFile
            //with a function with a stmtlist without braces
            let stmtlist = &mut node.children[0] //function
                .children[3] //blockexpr
                .children[0]; //stmtlist
            if stmtlist.children.len() == 0 {
                //handles empty type patches
                //Either no type or only minus
                return;
            }
            stmtlist.children = vec![stmtlist.children[0] //letstmt
                .children[3]
                .clone()] //Type;
        }
        striplet_aux(&mut self.plus);
        striplet_aux(&mut self.minus);
    }

    pub fn tagplus(&mut self) {
        fn tagplus_aux(node1: &mut Snode, node2: &Snode) {
            //There is no need to propagate pluses
            //because a plus cannot exist without code around it
            //when a '+' mod is written an ast is pushed at that
            //very level in the tree. That is I cannot write a plus
            //statement after a minus or context code and not have it
            //in a level same as the statement above it even around braces
            let mut pvec: Vec<Snode> = vec![];
            let mut achildren = node1.children.iter_mut();
            let mut bchildren = node2.children.iter();
            let mut a = achildren.next();
            let mut b = bchildren.next();
            loop {
                match (&mut a, &b) {
                    (Some(ak), Some(bk)) => {
                        match (ak.wrapper.modkind, bk.wrapper.modkind) {
                            (_, Some(MODKIND::PLUS)) => {
                                pvec.push((*bk).clone());
                                b = bchildren.next();
                            }
                            (Some(MODKIND::MINUS), _) => {
                                //minus code
                                //with any thing other than a plus
                                attachfront(ak, pvec);
                                pvec = vec![];
                                a = achildren.next();
                            }
                            (None, None) => {
                                //context code
                                //with context code
                                if ak.wrapper.isdisj {
                                    //DISJUNCTIONS ARE THE ONLY PART
                                    //WHERE PLUSES ARE ADDED TO A NODE
                                    //AND NOT A TOKEN
                                    ak.wrapper.plusesbef.extend(pvec);
                                } else {
                                    attachfront(ak, pvec);
                                }
                                pvec = vec![];
                                tagplus_aux(ak, bk);
                                a = achildren.next();
                                b = bchildren.next();
                            }
                            (Some(MODKIND::STAR), Some(MODKIND::STAR)) => {
                                a = achildren.next();
                                b = bchildren.next();
                            }
                            (Some(MODKIND::STAR), _) | (_, Some(MODKIND::STAR)) => {
                                panic!("Minus and Plus buffers do not have matching stars");
                            }
                            _ => {
                                panic!("There are plusses in the minus buffer, or minuses in the plus buffer.");
                            }
                        }
                    }
                    (None, Some(bk)) => match bk.wrapper.modkind {
                        Some(MODKIND::PLUS) => {
                            pvec.push((*bk).clone());
                            b = bchildren.next();
                        }
                        _ => {
                            break;
                        }
                    },
                    (Some(_), None) => {
                        break;
                    } //means only minuses are left
                    (None, None) => {
                        break;
                    }
                }
            }
            if pvec.len() != 0 {
                //Plus statements exist after
                //the context and need to be attached to the
                //closes context above
                let a = node1.children.last_mut();
                if a.is_none() {
                    panic!("Plus without context.");
                }
                let a = a.unwrap();
                if a.wrapper.isdisj {
                    a.wrapper.plusesaft.extend(pvec);
                } else {
                    //println!("attaching {} to {:?}", a.gettokenstream(), pvec);
                    attachback(a, pvec);
                }
            }
        }

        tagplus_aux(&mut self.minus, &mut self.plus);
    }

    pub fn getunusedmetavars(&self, mut bindings: Vec<MetaVar>) -> Vec<MetaVar> {
        let mut f = |x: &Snode| match &x.wrapper.metavar {
            MetaVar::NoMeta => {}
            MetaVar::Exp(info) | MetaVar::Id(info) | MetaVar::Type(info) => {
                if let Some(index) =
                    bindings.iter().position(|node| node.getname() == info.0.varname)
                //only varname is checked because a rule cannot have two metavars with same name but
                //different rulenames
                {
                    //this removes the metavaraible from the list of unused vars
                    //when encountered
                    bindings.remove(index);
                };
            }
        };

        collecttree(&self.minus, &mut f);
        collecttree(&self.plus, &mut f);
        debugcocci!("Unused Metavars:- {:?}", bindings);
        return bindings;
    }
}

pub struct Rule {
    pub name: Name,
    pub dependson: Dep,
    pub metavars: Vec<MetaVar>,
    pub unusedmetavars: Vec<MetaVar>,
    pub patch: Patch,
    pub freevars: Vec<MetaVar>,
    pub usedafter: HashSet<MetavarName>,
    pub hastype: bool,
}

// Given the depends clause it converts it into a Dep object
fn getdep(rules: &Vec<Rule>, lino: usize, dep: &mut Snode) -> Dep {
    dep.print_tree();
    match dep.kind() {
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
                    Dep::AndDep(Box::new((getdep(rules, lino, lhs), getdep(rules, lino, rhs))))
                }
                Tag::PIPE2 => {
                    //Recurses
                    Dep::OrDep(Box::new((getdep(rules, lino, lhs), getdep(rules, lino, rhs))))
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
fn handlerules(rules: &Vec<Rule>, decl: Vec<&str>, lino: usize) -> (Name, Dep, bool) {
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
    let fword = tokens.next();
    let fiword = tokens.next();

    let (depends, hastype) = match (sword, tword, fword, fiword) {
        (Some("depends"), Some("on"), Some(rule), hastype) => {
            let booleanexp: Name = rule.to_string();
            let hastype: bool = hastype.is_some_and(|x| x == "type");
            (getdependson(rules, Name::from(booleanexp).as_str(), lino), hastype)
        }
        (hastype, None, None, None) => (Dep::NoDep, hastype.is_some_and(|x| x == "type")),
        _ => syntaxerror!(lino, "Bad Syntax"),
    };

    (currrulename, depends, hastype)
}

/// Turns values from handlemods into a patch object
fn getpatch(
    plusbuf: &str,
    minusbuf: &str,
    llino: usize,
    metavars: &Vec<MetaVar>,
    hastype: bool,
) -> Patch {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    let mut p = Patch { plus: wrap_root(plusbuf.as_str()), minus: wrap_root(minusbuf.as_str()) };
    p.setmetavars(metavars);
    p.setmods();
    removestmtbraces(&mut p.minus);
    removestmtbraces(&mut p.plus);
    p.striplet(hastype);
    p.tagplus();
    p
}

/// Given all the info about name, depends, metavars and modifiers and context
/// it consolidates everything into a line preserved rule object
fn buildrule(
    currrulename: &Name,
    currdepends: Dep,
    mut metavars: Vec<MetaVar>, //mutable because unused metavars are removed
    blanks: usize,
    pbufmod: &String,
    mbufmod: &String,
    lastruleline: usize,
    hastype: bool,
) -> Rule {
    //end of the previous rule
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    plusbuf.push_str(format!("fn {currrulename}_plus() {{\n").as_str());
    minusbuf.push_str(format!("fn {currrulename}_minus() {{\n").as_str());

    plusbuf.push_str(&"\n".repeat(blanks));
    minusbuf.push_str(&"\n".repeat(blanks));

    if hastype {
        let pbufmod = if pbufmod.trim() != "" {
            format!("let COCCIVAR: \n{}\n;", pbufmod)
        } else {
            String::new()
        };

        let mbufmod = if mbufmod.trim() != "" {
            format!("let COCCIVAR: \n{}\n;", mbufmod)
        } else {
            String::new()
        };

        plusbuf.push_str(&pbufmod);
        minusbuf.push_str(&mbufmod);
    } else {
        plusbuf.push_str(pbufmod);
        minusbuf.push_str(mbufmod);
    }

    plusbuf.push_str("}");
    minusbuf.push_str("}");

    let currpatch = getpatch(&plusbuf, &minusbuf, lastruleline, &metavars, hastype);
    let unusedmetavars = currpatch.getunusedmetavars(metavars.clone());

    for metavar in &unusedmetavars {
        println!("Warning: Unused metavariable {}.{}", metavar.getrulename(), metavar.getname());
        if let Some(index) = metavars.iter().position(|x| x.getname() == metavar.getname()) {
            //All this will be optimised when using hashsets
            metavars.remove(index);
        }
    }

    let mut freevars: Vec<MetaVar> = vec![];
    for metavar in &metavars {
        if metavar.getrulename() != currrulename {
            debugcocci!("Ferevar found- {:?}", metavar);
            freevars.push(metavar.clone());
        }
    }

    let rule = Rule {
        name: Name::from(currrulename),
        dependson: currdepends,
        metavars: metavars,
        unusedmetavars: unusedmetavars,
        patch: currpatch,
        freevars: freevars,
        usedafter: HashSet::new(),
        hastype: hastype,
    };
    rule
}

/// Does nothing much as of now. Just appends lines inside the rules
/// while preserving line numbers with new lines
pub fn handlemods(block: &Vec<&str>) -> Result<(String, String, bool), (usize, String)> {
    let mut plusbuf = String::new();
    let mut minusbuf = String::new();
    let mut lino: usize = 0;
    let mut hasstar: bool = false;
    let mut hastforms: bool = false;

    for line in block {
        match line.chars().next() {
            Some('+') => {
                if hasstar {
                    return Err((
                        lino,
                        String::from("Transformations cannot be applied because of star prior"),
                    ));
                }
                hastforms = true;
                plusbuf.push_str(&format!("{}{}{}", "/*+*/", &line[1..], '\n'));
                minusbuf.push('\n');
            }
            Some('-') => {
                if hasstar {
                    return Err((
                        lino,
                        String::from("Transformations cannot be applied because of star prior"),
                    ));
                }
                hastforms = true;
                minusbuf.push_str(&format!("{}{}{}", "/*-*/", &line[1..], '\n'));
                plusbuf.push('\n');
            }
            Some('(') => {
                let holder = "if COCCIVAR {\n";
                plusbuf.push_str(holder);
                minusbuf.push_str(holder);
            }
            Some('|') => {
                let holder = "} else if COCCIVAR {\n";
                plusbuf.push_str(holder);
                minusbuf.push_str(holder);
            }
            Some(')') => {
                plusbuf.push_str("}\n");
                minusbuf.push_str("}\n");
            }
            Some('*') => {
                if hastforms {
                    return Err((
                        lino,
                        String::from("Star cannot be applied because of transformation prior"),
                    ));
                }
                hasstar = true;
                plusbuf.push_str(&format!("{}{}{}", "/***/", &line[1..], '\n'));
                minusbuf.push_str(&format!("{}{}{}", "/***/", &line[1..], '\n'));
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
    return Ok((plusbuf, minusbuf, hasstar));
}

/// Parses the metavar declarations
pub fn handle_metavar_decl(
    rules: &Vec<Rule>,
    block: &Vec<&str>,
    rulename: &Name,
    lino: usize,
) -> (Vec<MetaVar>, usize) {
    let mut offset: usize = 0;
    let mut blanks: usize = 0;
    let mut metavars: Vec<MetaVar> = vec![]; //stores the mvars encounteres as of now

    for line in block {
        offset += 1;
        let line = line.trim();
        if line == "" {
            continue;
        }
        let mut tokens = line.split(&[',', ' ', ';'][..]);
        let ty = tokens.next().unwrap().trim();
        for var in tokens {
            let var = var.trim().to_string();
            if var != "" {
                if !metavars.iter().any(|x| x.getname() == var) {
                    metavars.push(makemetavar(rules, rulename, &var, ty, lino));
                } else {
                    syntaxerror!(offset + lino, format!("Redefining {} metavariable {}", ty, var));
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

fn setusedafter(rules: &mut Vec<Rule>) {
    let mut tmp: HashSet<MetavarName> = HashSet::new();
    for rule in rules.iter_mut().rev() {
        rule.usedafter = tmp.clone();
        for freevar in &rule.freevars {
            tmp.insert(MetavarName {
                rulename: freevar.getrulename().to_string(),
                varname: freevar.getname().to_string(),
            });
        }
    }
}

pub fn processcocci(contents: &str) -> (Vec<Rule>, bool) {
    debugcocci!("{}", "Started Parsing");
    let mut blocks: Vec<&str> = contents.split("@").collect();
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut rules: Vec<Rule> = vec![];
    //check for empty
    if blocks.len() == 0 {
        return (vec![], false);
    }
    //handleprepatch(blocks.swap_remove(0)); //throwing away the first part before the first @
    handleprepatch(blocks.remove(0));
    let nrules = blocks.len() / 4; //this should always be an integer if case of a proper cocci file
                                   //if it fails we will find out in the next for loop

    let mut lastruleline = 0;
    let mut hasstars = false; //this does not ensure that all rules have only * or none
                              //TODO enforce that
    for i in 0..nrules {
        debugcocci!("Processing rule {}", i);
        let block1: Vec<&str> = blocks[i * 4].trim().lines().collect(); //rule
        let block2: Vec<&str> = blocks[i * 4 + 1].lines().collect(); //metavars
        let block3: Vec<&str> = blocks[i * 4 + 2].lines().collect(); //empty
        let block4: Vec<&str> = blocks[i * 4 + 3].lines().collect(); //mods

        //getting rule info
        let (currrulename, currdepends, hastype) = handlerules(&rules, block1, lino);
        debugcocci!("Rulename: {} Depends on: {:?}", currrulename, currdepends);
        lino += 1;
        let (metavars, blanks) = handle_metavar_decl(&rules, &block2, &currrulename, lino);
        //metavars
        lino += block2.len();

        if block3.len() != 0 {
            syntaxerror!(lino, "Syntax Error");
        }

        //modifiers
        lino += block4.len() - 1;
        let (pbufmod, mbufmod, hasstar) = handlemods(&block4).unwrap_or_else(|(lino, msg)| {
            syntaxerror!(lino, 0, msg);
        });
        if hasstar {
            hasstars = true;
            //This part needs to be completed
            //as it does not enforce all rules having *
        }

        let rule = buildrule(
            &currrulename,
            currdepends,
            metavars,
            blanks,
            &pbufmod,
            &mbufmod,
            lastruleline,
            hastype,
        );
        rules.push(rule);

        lastruleline = lino;
    }
    setusedafter(&mut rules);
    (rules, hasstars)
    //flag_logilines(0, &mut root);
}
