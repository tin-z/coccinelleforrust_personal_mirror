// Remove rules that cannot match due to false dependencies
// This should happen before free_vars
// For full Coccinelle, this will be more complex, due to disjunctions
// and <... ...>.  Disjunctions are ok if at least one branch is ok.
// <... ...> should be converted to ... if the pattern cannot match,
// but this doesn't amount to a failure.
// Removing unmatchable rules gives more freedom for metavariable
// matching when virtual rules are used to combine multiple semantic
// patches.

use std::vec;
use std::collections::HashSet;

use parser::SyntaxKind;

use crate::{
    util::{worktree},
    wrap::{metatype,Rnode},
    parse_cocci::{dep,mvar,rule}
};

type Tag = SyntaxKind;
type Name = String;

#[feature(box_patterns)]
fn executable(dropped: &HashSet<Name>, dep: &dep) -> bool {
    match dep {
        dep::NoDep => true,
        dep::FailDep => false,
        dep::Dep(name) => !dropped.contains(name),
        dep::AndDep(box(dep1, dep2)) =>
            executable(dropped, dep1) && executable(dropped, dep2),
        dep::OrDep(box(dep1, dep2)) =>
            executable(dropped, dep1) || executable(dropped, dep2),
        dep::AntiDep(_) => true // no idea if dep succeeds if executable
    }
}

fn bindable(dropped: &HashSet<Name>, minus: &Rnode) -> bool {
    let mut is_bindable = true;
    let work =
        |node: &Rnode| {
            if let Tag::PATH_EXPR = node.astnode.kind() {
                match node.wrapper.metatype {
                    metatype::NoMeta  => {}
                    metatype::Exp(mv) | metatype::Id(mv) => {
                        if dropped.contains(mv.rulename) {
                            is_bindable = false
                        }
                    }
                }
            };
        };
    worktree(minus, &work);
    is_bindable
}

pub fn cleanup_rules(rules: &mut Vec<rule>) {
    let dropped = HashSet::new();

    for rule in rules {
        if !executable(&dropped, rule.name) || !bindable(&dropped, rule.patch.minus) {
            dropped.insert(rule.name);
            rules.remove(rule);
        }
    }
}
