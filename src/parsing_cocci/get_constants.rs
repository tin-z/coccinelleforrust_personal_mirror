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
