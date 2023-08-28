// SPDX-License-Identifier: GPL-2.0

// The input should be in CNF, ie an outer list, representing a conjunction,
// with inner lists of words, representing disjunctions.  There is no negation.

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::vec::Vec;
use regex::Regex;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn interpret<'a>(mut big_regexp : &'a Regex, regexps : Vec<&'a Regex>, file : String) -> bool {
    if let Ok(lines) = read_lines(file) {
        let mut simple = regexps.len() == 1;
        for line in lines {
            if let Ok(l) = line {
                if big_regexp.is_match(&l) {
                    if simple {
                        return true
                    }
                    else {
                        let res: Vec<_> =
                            regexps.iter()
                                .filter(|regexp| !regexp.is_match(&l)).collect();
                        let rlen = res.len();
                        if rlen == 0 {
                            return true
                        }
                        else if rlen == 1 {
                            simple = true;
                            big_regexp = regexps[0];
                        }
                    }
                }
            }
        }
    }
    false
}
