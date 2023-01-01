// For each rule return the list of variables that are used after it.
// Also augment various parts of each rule with unitary, inherited, and freshness
// information

use std::vec;
use std::collections::HashSet;
use parser::SyntaxKind;

use crate::wrap::{Rnode,metatype};
use crate::parse_cocci::{mvar,rule};
use crate::util::{worktree};

type Tag = SyntaxKind;
type Name = String;

// ----------------------------------------------------------------
// Basic collection functions for a single rule.
// 

fn collect_unitary_nonunitary(free_usage: &Vec<mvar>) -> (HashSet<mvar>,HashSet<mvar>) {
    let mut unitary: HashSet<mvar>    = HashSet::new();
    let mut nonunitary: HashSet<mvar> = HashSet::new();
    for id in free_usage {
        let id = id.clone();
        if unitary.contains(id) {
            unitary.remove(id);
            nonunitary.insert(id);
        }
        else {
            if !nonunitary.contains(id) {
                unitary.insert(id);
            }
        }
    }
    (unitary,nonunitary)
}

fn collect_refs(root: Rnode, add: &dyn Fn(&mvar)) {
    let work =
        |node: Rnode| {
            if let Tag::PATH_EXPR = node.astnode.kind() {
                match node.wrapper.metatype {
                    metatype::NoMeta  => {}
                    metatype::Exp(mv) | metatype::Id(mv) => add(&mv.clone())
                }
            }
        };
    worktree(root, &work)
}

fn collect_minus_refs_unitary_nonunitary(root: Rnode) -> (HashSet<mvar>,HashSet<mvar>) {
    let mut refs: Vec<mvar> = vec![];
    let add = &|x| refs.push(x);
    collect_refs(root, add);
    let mut allrefs: HashSet<mvar> = HashSet::new();
    collect_unitary_nonunitary(refs)
}

fn collect_plus_refs(root: Rnode) -> HashSet<mvar> {
    let mut refs: HashSet<mvar> = HashSet::new();
    let add = &|x| refs.insert(x);
    let work_exp_list_list =
        |expss: Vec<Vec<Rnode>>| {
            for x in expss {
                for y in x {
                    collect_refs(y, add)
                }
            }
        };
    let work =
        |node| {
            match node.wrapper.mcodekind {
                MINUS(REPLACEMENT(rexpss)) =>
                    work_exp_list_list(rexpss),
                MINUS(NOREPLACEMENT) => {}
                CONTEXT(BEFORE(beforeexpss)) =>
                    work_exp_list_list(beforeexpss),
                CONTEXT(AFTER(afterexpss)) =>
                    work_exp_list_list(afterexpss),
                CONTEXT(BEFOREAFTER(beforeexpss,afterexpss)) => {
                    work_exp_list_list(beforeexpss);
                    work_exp_list_list(afterexpss)
                }
                CONTEXT(NOTHING) => {}
                _ => error("unexpected PLUS or MIXED in free_vars::collect_plus_refs",
                           node)
            }
        };
    worktree(root, &work)
}

// ----------------------------------------------------------------
// classify as unitary (no binding) or nonunitary (env binding) or saved
// (witness binding)

fn classify_rule_variables(rule: &mut rule, used_after: &mut Vec<mvar>) {
    let curname = rule.name;

    let (unitarynames,nonunitarynames) = collect_minus_refs_unitary_nonunitary(rule.patch.minus);
    let inplus = collect_plus_refs(rule.patch.minus);

    let mut saved:      HashSet<mvar> = inplus;
    let mut unitary:    HashSet<mvar> = HashSet::new();
    let mut nonunitary: HashSet<mvar> = HashSet::new();

    // classify metavariables as saved, unitary, and nonunitary
    let check_nonunitary =
        |r| if used_after.contains(r) {
                saved.push(r);
            }
            else {
                nonunitary.push(r);
            };

    for r in unitarynames {
        if r.rulename == curname &&
           !inplus.contains(r) && !used_after.contains(r) {
            unitary.push(r);
        }
        else {
            check_nonunitary(r);
        }
    }

    for r in nonunitarynames {
        check_nonunitary(r);
    }

    rule.saved = saved;
    rule.unitary = unitary;
    rule.nonunitary = nonunitary;

    // collect the nonlocal and inherited variables
    let free_vars = unitary.clone();
    let inherited = vec![];

    let collect =
        |r| if r.rulename == curname {
                free_vars.push(r);
            }
            else {
                inherited.push(r);
            };

    for r in saved {
        collect(r);
    }

    for r in nonunitary {
        collect (r);
    }

    // drop the local variables from used_after
    for r in used_after {
        if r.rulename == curname {
            used_after.remove(&r);
        }
    }

    // add the nonlocal variables to used_after
    let collect =
        |r| if r.rulename != curname {
                used_after.insert(&r);
            };

    for r in saved {
        collect(&r);
    }

    for r in nonunitary {
        collect(&r);
    }
}

// ----------------------------------------------------------------
// ----------------------------------------------------------------
// entry point

pub fn free_vars(rules: &mut Vec<rule>) {
    let mut used_after: Vec<mvar> = vec![];

    for rule in rules.iter().rev() {
        classify_rule_variables(rule, &used_after);
    }
}
