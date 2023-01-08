/*
(*
 * This file is part of Coccinelle, licensed under the terms of the GPL v2.
 * See copyright.txt in the Coccinelle source code for more information.
 * The Coccinelle source code can be obtained at http://coccinelle.lip6.fr
 *)

(* Issues:

1.  If a rule X depends on a rule Y (in a positive way), then we can ignore
    the constants in X.

2.  If a rule X contains a metavariable that is not under a disjunction and
    that is inherited from rule Y, then we can ignore the constants in X.

3.  If a rule contains a constant x in + code then subsequent rules that
    have it in - or context should not include it in their list of required
    constants.
*)

(* This doesn't do the . -> trick of get_constants for record fields, as
    that does not fit well with the recursive structure.  It was not clear
    that that was completely safe either, although eg putting a newline
    after the . or -> is probably unusual. *)
*/

use regex::Regex;
use std::collections::BTreeSet;
use std::ops::Deref;
use crate::{syntaxerror, commons::util};

// -----------------------------------------------------------------------
// This phase collects everything.  One can then filter out what it not
// wanted

// True means nothing was found
// False should never drift to the top, it is the neutral element of or
// and an or is never empty

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Combine {
    True,
    False,
    Elem(String),
    And(Box<BTreeSet<Combine>>),
    Or(Box<BTreeSet<Combine>>),
    Not(Box<Combine>),
}
use Combine::*;


static false_on_top_err: &str =
    &"No rules apply.  Perhaps your semantic patch doesn't contain any +/-/* code, or you have a failed dependency.";

fn str_concat_fn<T>(lst: &BTreeSet<T>, op: &dyn Fn(&T) -> String, bet: &str) -> String {
    let strs : Vec<String> = lst.into_iter().map(|x| op(x)).collect();
    strs.join(format!(" {bet} ").as_str())
}

fn dep2c (dep: &Combine) -> String {
    match dep {
        And(l) => format!("({})", str_concat_fn(&l, &dep2c, &"&")),
        Or(l) => format!("({})", str_concat_fn(&l, &dep2c, &"|")),
        Not(x) => format!("!({})", dep2c(x)),
        Elem(x) => x.to_string(),
        False => String::from("false"),
        True => String::from("true")
    }
}

// ---------------------------------------------------------------------------------------
// interpretation for use with grep
// grep only does or

fn interpret_grep(strict: bool, x: &Combine) -> Option<BTreeSet<String>> {
    fn rec (collected: &mut BTreeSet<String>, strict: bool, cmb: &Combine) {
        match cmb {
            Elem(x) => { collected.insert(x.to_string()); }
            Not(_) => syntaxerror!(0, "not unexpected in grep arg"),
            And(l) | Or(l) =>
                for x in l.iter() {
                    rec(collected, strict, x);
                },
            True =>
                if strict {
                    syntaxerror!(0, "True should not be in the final result")
                }
                else {
                    collected.insert(String::from("True"));
                },
            False =>
                if strict {
                    syntaxerror!(0, false_on_top_err)
                }
                else {
                    collected.insert(String::from("False"));
                }
        }
    }
    match x {
        True => None,
        False if strict =>
            syntaxerror!(0, false_on_top_err),
        _ => {
            let mut collected = BTreeSet::new();
            rec(&mut collected, strict, x);
            Some(collected)
        }
    }
}

// ---------------------------------------------------------------------------------------
// interpretation for use with git grep

// convert to cnf, give up if the result is too complex
static max_cnf: usize = 5;

fn opt_union_set(longer: &mut BTreeSet<BTreeSet<String>>, shorter: BTreeSet<BTreeSet<String>>) {
    for x in shorter {
        if !(longer.iter().any(|y| x.is_subset(y))) {
            longer.insert(x);
        }
    }
}

fn mk_false() -> BTreeSet<BTreeSet<String>> {
    BTreeSet::from([BTreeSet::new()])
}

fn cnf (strict: bool, dep: &Combine) -> Result<BTreeSet<BTreeSet<String>>,()> {
    match dep {
        Elem(x) => Ok(BTreeSet::from([BTreeSet::from([x.to_string()])])),
        Not(_) => syntaxerror!(0, "not unexpected in coccigrep arg"),
        And(l) => {
            let l = l.deref();
            if l.is_empty() {
                syntaxerror!(0, "and should not be empty")
            }
            let mut res = BTreeSet::new();
            for x in l.iter() {
                opt_union_set(&mut res, cnf(strict, x)?)
            }
            Ok(res)
        }
        Or(l) => {
            let l = l.deref();
            let mut ors = Vec::new();
            for x in l {
                ors.push(cnf(strict, x)?)
            }
            let icount = ors.iter().filter(|x| x.len() <= 1).count();
            if icount > max_cnf {
                Err(())
            }
            else {
                if ors.len() == 0 {
                    Ok(mk_false())
                }
                else {
                    let fst = ors.swap_remove(0);
                    let mut prev = fst.clone();
                    for cur in ors {
                        let curval: Vec<BTreeSet<BTreeSet<String>>> =
                            cur.iter().map(|x| {
                                prev.iter().map(|y| {
                                    x.union(&y).cloned().collect()
                                }).collect()
                            }).collect();
                        // drain prev
                        prev.clear();
                        // prev is now empty; reuse it
                        for x in curval {
                            opt_union_set(&mut prev, x);
                        }
                    }
                    Ok(prev)
                }
            }
        }
        True => Ok(BTreeSet::new()),
        False => {
            if strict {
                syntaxerror!(0, false_on_top_err)
            }
            else {
                Ok(mk_false())
            }
        }
    }
}

/*
fn interpret_cocci_git_grep (strict: bool, x: &Combine) -> Option<(Regex, Vec<Regex>, Vec<String>)> {
    fn optimize (l : BTreeSet<BTreeSet<String>>) -> BTreeSet<BTreeSet<String>> {
        let l = l.iter().map(|x| (x.len(), x)).collect();
        let l = l.sort().reverse().map(|(_,x)| x).collect;
        let mut res = BTreeSet::<BTreeSet<String>>::new();
        for cur in l {
            if !res.any(|x| cur.is_subset(x)) {
                res.insert(cur)
            }
        }
        res
    }
    fn atoms (dep: &Combine) -> BTreeSet<String> {
        fn rec (dep: &Combine, acc: BTreeSet<String>) {
            match dep {
                Elem(x) => { acc.insert(x); }
                Not(x) => syntaxerror!(0, "not unexpected in atoms"),
                And(l) | Or(l) => {
                    for x in l.deref() {
                        rec(x, acc)
                    }
                }
                True | False => {}
            }
        }
        let acc = BTreeSet::new();
        rec(dep, acc)
    }
    fn wordify(x: &String) -> String {
        format!("\\b{}\\b", x.to_string())
    }
    match x {
        True => None,
        False if strict => syntaxerror!(0, false_on_top_err),
        _ => {
            let resfn = || { // allow use of ?
                fn orify(l: BTreeSet<String>) -> Regex {
                    let str = str_concat_fn(l, wordify, &"\\|");
                    Regex::new(str.as_str()).unwrap()
                }
                let res1: Regex = orify(atoms(&x)); // all atoms
                let res = cnf(strict, x)?;
                let res = optimize(res);
                let res = /*Cocci_grep.split*/ res; // Must fix!!!
                let res2: Vec<Regex> = res.iter().map(orify).collect(); // atoms in conjunction
                let res3: Vec<String> =
                    res.iter().map(|x| format!("\\( -e {} \\)", x.join(" -e "))).collect();
                Ok((res1,res2,res3))
            };
            match resfn() {
                Ok(x) => Some(x),
                Err(_) => None
            }
        }
    }
}
*/
