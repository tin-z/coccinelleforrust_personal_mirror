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
use std::{collections::HashSet, ops::Deref, vec};

use super::ast0::{wrap_root, MetaVar, Snode, MODKIND, MetavarName};
use crate::{
    commons::util::{
        self, attachback, attachfront, collecttree, removestmtbracesaddpluses,
        worksnode
    },
    syntaxerror,
};
use parser::SyntaxKind;

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
        (Some(var), None, None) => MetaVar::new(rulename, var, metatype),
        (Some(rulen), Some(var), None) => {
            let var = var.deref();
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
    fn setmetavars_aux(node: &mut Snode, metavars: &Vec<MetaVar>) {
        let mut work = |node: &mut Snode| match node.kind() {
            Tag::PATH_EXPR | Tag::IDENT_PAT | Tag::NAME_REF => {
                let stmp = node.astnode.to_string();
                if let Some(mvar) = metavars.iter().find(|x| x.getname() == stmp) {
                    //println!("MetaVar found - {:?}", mvar);
                    node.wrapper.metavar = mvar.clone();
                }
            }
            _ => {}
        };
        util::worktree(node, &mut work);
    }

    fn setmetavars(&mut self, metavars: &Vec<MetaVar>) {
        Patch::setmetavars_aux(&mut self.plus, metavars);
        Patch::setmetavars_aux(&mut self.minus, metavars);
    }

    fn setmods(&mut self) {
        let mut tagmods = |node: &mut Snode,
                           (lino, modkind): (usize, Option<MODKIND>)|
         -> (usize, Option<MODKIND>) {
            let (start, end) = node.wrapper.getlinenos();

            match node.wrapper.modkind {
                Some(modkind) => {
                    if start == end {
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
                            pvec.push(bk.deref().clone());
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
                            Patch::tagplus_aux(ak, bk);
                            a = achildren.next();
                            b = bchildren.next();
                        }
                        _ => {
                            panic!("There are plusses in the minus buffer, or minuses in the plus buffer.");
                        }
                    }
                }
                (None, Some(bk)) => match bk.wrapper.modkind {
                    Some(MODKIND::PLUS) => {
                        pvec.push(bk.deref().clone());
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
                attachback(a, pvec);
            }
        }
    }

    pub fn tagplus(&mut self) {
        Patch::tagplus_aux(&mut self.minus, &mut self.plus);
    }

    pub fn getunusedmetavars(&self, mut bindings: Vec<MetaVar>) -> Vec<MetaVar> {
        let mut f = |x: &Snode| match &x.wrapper.metavar {
            MetaVar::NoMeta => {}
            MetaVar::Exp(info) | MetaVar::Id(info) => {
                if let Some(index) = bindings.iter().position(|node| node.getname() == info.0.varname)
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
fn getpatch(plusbuf: &str, minusbuf: &str, llino: usize, metavars: &Vec<MetaVar>) -> Patch {
    let plusbuf = format!("{}{}", "\n".repeat(llino), plusbuf);
    let minusbuf = format!("{}{}", "\n".repeat(llino), minusbuf);
    let mut p = Patch { plus: wrap_root(plusbuf.as_str()), minus: wrap_root(minusbuf.as_str()) };
    p.setmetavars(metavars);
    p.setmods();
    removestmtbracesaddpluses(&mut p.minus);
    removestmtbracesaddpluses(&mut p.plus);
    p.tagplus();
    p
}

/// Given all the info about name, depends, metavars and modifiers and context
/// it consolidates everything into a line preserved rule object
fn buildrule(
    currrulename: &Name,
    currdepends: Dep,
    mut metavars: Vec<MetaVar>,
    blanks: usize,
    pbufmod: &String,
    mbufmod: &String,
    lastruleline: usize,
) -> Rule {
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

    let currpatch = getpatch(&plusbuf, &minusbuf, lastruleline, &metavars);
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
                plusbuf.push_str("/*+*/");
                plusbuf.push_str(&line[1..]);
                plusbuf.push('\n');
                minusbuf.push('\n');
            }
            Some('-') => {
                minusbuf.push_str("/*-*/");
                minusbuf.push_str(&line[1..]);
                minusbuf.push('\n');
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
        if line.deref() == "" {
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

pub fn processcocci(contents: &str) -> Vec<Rule> {
    let mut blocks: Vec<&str> = contents.split("@").collect();
    let mut lino = 0; //stored line numbers
                      //mutable because I supply it with modifier statements

    let mut rules: Vec<Rule> = vec![];
    //check for empty
    if blocks.len() == 0 {
        return vec![];
    }
    //handleprepatch(blocks.swap_remove(0)); //throwing away the first part before the first @
    handleprepatch(blocks.remove(0));
    let nrules = blocks.len() / 4; //this should always be an integer if case of a proper cocci file
                                   //if it fails we will find out in the next for loop

    let mut lastruleline = 0;
    for i in 0..nrules {
        let block1: Vec<&str> = blocks[i * 4].trim().lines().collect(); //rule
        let block2: Vec<&str> = blocks[i * 4 + 1].lines().collect(); //metavars
        let block3: Vec<&str> = blocks[i * 4 + 2].lines().collect(); //empty
        let block4: Vec<&str> = blocks[i * 4 + 3].lines().collect(); //mods

        //getting rule info
        let (currrulename, currdepends) = handlerules(&rules, block1, lino);

        lino += 1;
        let (metavars, blanks) = handle_metavar_decl(&rules, &block2, &currrulename, lino);
        //println!("lino1 - {}", lino);
        //metavars
        lino += block2.len();
        //println!("lino2 - {}", lino);
        //just checks that nothing exists between the two @@
        //println!("lineo3 - {:?}", block3);
        if block3.len() != 0 {
            syntaxerror!(lino, "Syntax Error");
        }

        //modifiers
        lino += block4.len() - 1;
        let (pbufmod, mbufmod) = handlemods(&block4);

        let rule = buildrule(
            &currrulename,
            currdepends,
            metavars,
            blanks,
            &pbufmod,
            &mbufmod,
            lastruleline,
        );
        rules.push(rule);

        lastruleline = lino;
    }
    setusedafter(&mut rules);
    rules
    //flag_logilines(0, &mut root);
}
