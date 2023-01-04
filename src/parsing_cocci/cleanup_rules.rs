// Remove rules that cannot match due to false Dependencies
// This should happen before free_vars
// For full Coccinelle, this will be more complex, due to disjunctions
// and <... ...>.  Disjunctions are ok if at least one branch is ok.
// <... ...> should be converted to ... if the pattern cannot match,
// but this doesn't amount to a failure.
// Removing unmatchable rules gives more freedom for metavariable
// matching when virtual rules are used to combine multiple semantic
// patches.

use std::{vec, ops::Deref};
use std::collections::HashSet;

use parser::SyntaxKind;
use crate::util::worktree;
use super::{
    wrap::{MetaVar,Rnode},
    parse_cocci::{Dep,Rule}
};

type Tag = SyntaxKind;
type Name = String;

fn executable(dropped: &HashSet<&Name>, Dep: &Dep) -> bool {
    match Dep {
        Dep::NoDep => true,
        Dep::FailDep => false,
        Dep::Dep(name) => !dropped.contains(name),
        Dep::AndDep(dep) => //dep is deconstructed into dep1 dep2
        {
            let (dep1, dep2) = dep.deref();
            executable(dropped, &dep1) && executable(dropped, &dep2)
        },  
        Dep::OrDep(dep) => //dep is deconstructed into dep1 dep2
        {
            let (dep1, dep2) = dep.deref();
            executable(dropped, &dep1) || executable(dropped, &dep2)
        },
        Dep::AntiDep(_) => true // no idea if Dep succeeds if executable
    }
}

fn bindable(dropped: &HashSet<&Name>, minus: &mut Rnode) -> bool {
    let mut is_bindable = true;
    let mut work =
        |node: &mut Rnode| {
            if let Tag::PATH_EXPR = node.astnode.kind() {
                match &node.wrapper.metavar {
                    MetaVar::NoMeta  => {}
                    MetaVar::Exp(mv) | MetaVar::Id(mv) => {
                        if dropped.contains(&mv.1) {
                            is_bindable = false
                        }
                    }
                }
            };
        };
    worktree(minus, &mut work);
    is_bindable
}

pub fn cleanup_rules(rules: &mut Vec<Rule>) {
    let mut dropped = HashSet::new();

    let mut ins = vec![];
    let mut ctr = 0;
    for rule in rules.iter_mut() {
        if !executable(&dropped, &rule.dependson) || !bindable(&dropped, &mut rule.patch.minus) {
            dropped.insert(&rule.name);
            ins.push(ctr);
        }
        ctr+=1;
    }

    for i in 0..ins.len(){
        rules.remove(ins[i] - i);
    }
}
