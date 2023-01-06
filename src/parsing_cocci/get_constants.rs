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
        Combine::And(l) => format!("({})", str_concat_fn(l.deref(), &|x| dep2c(x), &"&")),
        Combine::Or(l)  => format!("({})", str_concat_fn(l.deref(), &|x| dep2c(x), &"|")),
        Combine::Not(x) => format!("!({})", dep2c(x)),
        Combine::Elem(x) => x.to_string(),
        Combine::False => String::from("false"),
        Combine::True => String::from("true")
    }
}

// grep only does or
fn interpret_grep(strict: bool, x: &Combine) -> Option<BTreeSet<&String>> {
    fn rec<'a> (collected: &mut BTreeSet<&'a String>, strict: bool, cmb: &'a Combine) {
        match cmb {
            Combine::Elem(x) => { collected.insert(x); }
            Combine::Not(_) => syntaxerror!(0, "not unexpected in grep arg"),
            Combine::And(l) | Combine::Or(l) =>
                for x in l.iter() {
                    rec(collected, strict, x);
                },
            Combine::True =>
                if strict {
                    syntaxerror!(0, "True should not be in the final result")
                }
                else {
                    collected.insert(&String.from("True"));
                },
            Combine::False =>
                if strict {
                    syntaxerror!(0, false_on_top_err)
                }
                else {
                    collected.insert(&String.from("False"));
                }
        }
    }
    match x {
        Combine::True => None,
        Combine::False if strict =>
            syntaxerror!(0, false_on_top_err),
        _ => {
            let mut collected = BTreeSet::new();
            rec(&mut collected, strict, x);
            Some(collected)
        }
    }
}

static max_cnf: i32 = 5;

fn interpret_cocci_git_grep (strict: bool, x: Combine) -> Option<Combine> {
    // convert to cnf
    fn opt_union_set(mut longer: BTreeSet<BTreeSet<String>>, shorter: BTreeSet<BTreeSet<String>>) {
        for x in shorter {
            if !longer.iter().any(|y| x.is_subset(y)) {
                longer.insert(x)
            }
        }
    }
    fn mk_false() -> BTreeSet<BTreeSet<String>> {
        BTreeSet::from([BTreeSet::new()])
    }
    fn cnf (strict: bool, dep: Combine) -> Result<BTreeSet<BTreeSet<String>>,()> {
        match dep {
            Combine::Elem(x) => Ok(BTreeSet::from([BTreeSet::from([x])])),
            Combine::Not(x) => syntaxerror!(0, "not unexpected in coccigrep arg"),
            Combine::And(l) => {
                let l = l.deref();
                if l.is_empty() {
                    syntaxerror!(0, "and should not be empty")
                }
                let mut res = BTreeSet::new();
                l.iter().for_each(|x| opt_union_set(res, cnf(strict, x)));
                Ok(res)
            }
            Combine::Or(l) => {
                let l = l.deref();
                let l = l.iter().map(|x| cnf(strict, x)).collect();
                let icount = l.iter().filter(|x| x.len <= 1).count();
                if icount > max_cnf {
                    Err(())
                }
                else {
                    if l.is_empty() {
                        Ok(mk_false())
                    }
                    else {
                        let mut res = BTreeSet::new();
                        for x in l {
                            let mut innerres = BTreeSet::new();
                            for y in x {
                                for z in res {
                                    innerres.insert(z.union(y))
                                }
                            }
                            opt_union_set(res, innerres)
                        }
                        Ok(res)
                    }
                }
            }
            Combine::True => Ok(BTreeSet::new()),
            Combine::False => {
                if strict {
                    syntaxerror!(0, false_on_top_err)
                }
                else {
                    Ok(mk_false())
                }
            }
        }
    }
    fn optimize (l : BTreeSet<BTreeSet<String>>) {
        let l = l.iter().map(|x| (x.len(), x)).collect();
        let l = l.sort().reverse().map(|(_,x)| x).collect;
        let mut res = BTreeSet<BTreeSet<String>>::new();
        for cur in l {
            if !res.any(|x| cur.is_subset(x)) {
                res.insert(cur)
            }
        }
    }
    fn atoms (dep: &Combine) -> BTreeSet<String> {
        fn rec (dep: &Combine, acc: BTreeSet<String>) {
            match dep {
                Combine::Elem(x) => acc.insert(x),
                Combine::Not(x) => syntaxerror!(0, "not unexpected in atoms"),
                Combine::And(l) | Combine::Or(l) => {
                    for x in l {
                        rec(x, acc)
                    }
                }
                Combine::True | Combine::False => {}
            }
        }
        let acc = BTreeSet::new();
        rec(dep, acc)
    }
    fn wordify(x: String) -> String {
        format!("\\b{x}\\b")
    }
    match x {
        True => None,
        False if strict => syntaxerror!(0, false_on_top_err),
        _ => {
            let res = {
                fn orify(l: BTreeSet<String>) -> String {
                    let str = str_concat_fn(l, wordify, &"\\|");
                    Regex::new(str.as_str()).unwrap()
                }
                let res1 = orify(atoms(&x)); // all atoms
                let res = cnf(strict, x)?;
                let res = optimize(res);
                let res = /*Cocci_grep.split*/ res; // Must fix!!!
                let res2 = res.map(orify).collect(); // atoms in conjunction
                let res3 =
                    res.map(|x| format!("\\( -e {} \\)", x.join(" -e "))).collect();
                Ok((res1,res2,res3))
            };
            match res {
                Ok(x) => Some(x),
                Err(_) => None
            }
        }
    }
}
