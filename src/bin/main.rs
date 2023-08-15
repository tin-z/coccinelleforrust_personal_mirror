// SPDX-License-Identifier: GPL-2.0

use clap::Parser;
use coccinelleforrust::commons::info::ParseError::{self, *};
use coccinelleforrust::parsing_rs::parse_rs::processrs;
use coccinelleforrust::{
    engine::cocci_vs_rs::MetavarBinding, engine::transformation,
    interface::interface::CoccinelleForRust, parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode,
};
use env_logger::{Builder, Env};
use itertools::{Itertools, izip};
use rand::Rng;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::fs::DirEntry;
use std::io;
use std::io::Write;
use std::process::Command;
use std::{fs, path::Path, process::exit};

#[allow(dead_code)]
fn tokenf<'a>(_node1: &'a Snode, _node2: &'a Rnode) -> Vec<MetavarBinding> {
    // this is
    // Tout will have the generic types in itself
    // ie  ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn init_logger(args: &CoccinelleForRust) {
    let mut options = String::new();
    if args.debug {
        options.push_str(
            "
            coccinelleforrust::parsing_cocci,
            coccinelleforrust::commons,
            coccinelleforrust::engine
        ",
        );
    }
    let env = Env::default().default_filter_or(&options);

    Builder::from_env(env)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();
}


pub fn adjustformat(node1: &mut Rnode, node2: &Rnode, mut line: Option<usize>) -> Option<usize> {
    
    if node1.wrapper.wspaces.0.contains("/*COCCIVAR*/") {
        node1.wrapper.wspaces = node2.wrapper.wspaces.clone();

        line = Some(node1.wrapper.info.sline);
    }

    for (childa, childb) in izip!(&mut node1.children, &node2.children) {
        line.map(|sline|{
            if childa.wrapper.info.eline==sline {
                childa.wrapper.wspaces = childb.wrapper.wspaces.clone();
            }
            else {
                line = None;
            }
        });

        line = adjustformat(childa, &childb, line);
    }

    return line;

}

/// Get the formatted contents and diff of a file
///
/// ## Arguments
///
///  - `cfr`: Configuration object containing arguments provided by the user
///  - `transformedcode`: Mutable reference to a node of modified Rust code
///  - `targetpath`: Output path of the file to be written
///
/// ## Return Value
///
/// This function returns a tuple of [`String`]s, which represent respectively
/// the formatted content of the file, and the delta or 'diff' resulting from
/// the file prior to modification.
fn getformattedfile(
    cfr: &CoccinelleForRust,
    transformedcode: &mut Rnode,
    targetpath: &str,
) -> (String, String) {
    let mut rng = rand::thread_rng();
    let randrustfile = format!("/tmp/tmp{}.rs", rng.gen::<u32>());

    // In all cases, write the transformed code to a file so we can diff
    transformedcode.writetreetofile(&randrustfile);

    // Now, optionally, we may want to not rust-format the code.
    if !cfr.suppress_formatting {
        //should never be disabled except for debug
        //let original = fs::read_to_string(&targetpath).expect("Unable to read file");

        // Depending on whether the user provided a `rustfmt` config file or not,
        // add it to the invokation.
        // As it turns out, argument order for rustfmt does not matter, you can
        // add the configuration file later and it is still accounted for when
        // parsing files provided before.

        // Core command is in a separate binding so it stays alive. The others
        // are just references to it.
        let mut core_command = Command::new("rustfmt");
        let mut fcommand = core_command
            .arg(&randrustfile)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if let Some(fmtconfig_path) = &cfr.rustfmt_config {
            fcommand = fcommand.arg("--config-path")
                .arg(fmtconfig_path.as_str());
        }

        if fcommand.spawn().expect("rustfmt failed").wait().is_err() {
            println!("Formatting failed.");
        }

        let formattednode = processrs(&fs::read_to_string(&randrustfile).expect("Could not read")).unwrap();
        adjustformat(transformedcode, &formattednode, None);
        transformedcode.writetreetofile(&randrustfile);

        //fs::write(&randrustfile, original.clone()).expect("Could not write file.");
    }

    let diffed = if !cfr.suppress_diff {
        let diffout = Command::new("git")
            .arg("diff")
            .arg("--no-index")
            .arg("--diff-algorithm=histogram")
            .arg(targetpath)
            .arg(&randrustfile)
            .output()
            .expect("diff failed");

        String::from_utf8(diffout.stdout).expect("Bad diff")
    } else {
        String::new()
    };

    let formatted = fs::read_to_string(&randrustfile).expect("Unable to read file");

    //fs::write(targetpath, original).expect("Could not write file.");
    fs::remove_file(&randrustfile).expect("No file found.");

    //}

    //transformedcode.writetreetofile(&randrustfile);
    //let transformed = fs::read_to_string(&randrustfile).expect("Could not read generated file");
    //fs::remove_file(randrustfile).expect("Could not reove file");

    (formatted, diffed)
}

fn transformfiles(args: &CoccinelleForRust, files: Vec<String>) {
    let failedfiles: Vec<(ParseError, usize)> = vec![];

    let transform = &|targetpath: &String| {
        println!("Processing: {}", targetpath);

        let patchstring =
            fs::read_to_string(args.coccifile.as_str()).expect("This shouldnt be empty");
        let rustcode = fs::read_to_string(targetpath.as_str()).expect("This shouldnt be empty");

        let transformedcode = transformation::transformfile(patchstring, rustcode);

        let (mut transformedcode, hasstars) = match transformedcode {
            Ok(node) => node,
            Err(_error) => {
                //failedfiles.push((error, targetpath));
                println!("Failed to transform {}", targetpath);
                return;
            }
        };
        let (data, diff) = getformattedfile(&args, &mut transformedcode, &targetpath);
        if !hasstars {
            if !args.suppress_diff {
                println!("{}", diff);
            }

            if args.apply {
                fs::write(targetpath, data).expect("Unable to write")
            } else {
                if let Some(outputfile) = &args.output {
                    if let Err(written) = fs::write(outputfile, data) {
                        eprintln!("Error in writing file.\n{:?}", written);
                    }
                }
            }
        } else {
            println!("Code highlighted with *");
            for line in diff.split("\n").collect_vec() {
                if line.len() != 0 && line.chars().next().unwrap() == '-' {
                    print!("*{}\n", &line[1..]);
                } else {
                    print!("{}\n", line)
                }
            }
        }
    };

    if args.no_parallel {
        files.iter().for_each(transform);
    } else {
        files.par_iter().for_each(transform);
    }

    if failedfiles.len() == 0 {
        return;
    }

    println!("Failed transformations :- ");
    for (error, targetpath) in failedfiles {
        match error {
            TARGETERROR(errors, _) => {
                println!("Error in reading target file.\n{}", errors);
                println!("Unparsable file:\n{}", targetpath);
            }
            RULEERROR(rulename, errors, _) => {
                println!("Error in applying rule {}", rulename);
                println!("Error:\n{}", errors);
                println!("Unparsable file:\n{}", targetpath);
            }
        }

        println!();
    }
}

fn transformfile(args: &CoccinelleForRust) {
    let patchstring = fs::read_to_string(args.coccifile.as_str()).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(args.targetpath.as_str()).expect("This shouldnt be empty");

    let transformedcode = transformation::transformfile(patchstring, rustcode);

    let (mut transformedcode, hasstars) = match transformedcode {
        Ok(node) => node,
        Err(TARGETERROR(errors, _)) => {
            eprintln!("Error in reading target file.\n{}", errors);
            eprintln!("Unparsable file:\n{}", args.targetpath);
            panic!();
        }
        Err(RULEERROR(rulename, errors, _)) => {
            eprintln!("Error in applying rule {}", rulename);
            eprintln!("Error:\n{}", errors);
            eprintln!("Unparsable file:\n{}", args.targetpath);
            panic!();
        }
    };
    let (data, diff) = getformattedfile(&args, &mut transformedcode, &args.targetpath);
    if !hasstars {
        if !args.suppress_diff {
            println!("{}", diff);
        }

        if args.apply {
            fs::write(&args.targetpath, data).expect("Unable to write")
        } else {
            if let Some(outputfile) = &args.output {
                if let Err(written) = fs::write(outputfile, data) {
                    eprintln!("Error in writing file.\n{:?}", written);
                }
            }
        }
    } else {
        println!("Code highlighted with *");
        for line in diff.split("\n").collect_vec() {
            if line.len() != 0 && line.chars().next().unwrap() == '-' {
                print!("*{}\n", &line[1..]);
            } else {
                print!("{}\n", line)
            }
        }
    }
}

fn makechecks(args: &CoccinelleForRust) {
    if !Path::new(args.targetpath.as_str()).exists() {
        eprintln!("Target file/path does not exist.");
        exit(1);
    }

    if !Path::new(args.coccifile.as_str()).exists() {
        eprintln!("Semantic file/path does not exist.");
        exit(1);
    }
}

#[allow(dead_code)]
fn findfmtconfig(args: &mut CoccinelleForRust) {
    let height_lim: usize = 5;

    let mut target = Path::new(args.targetpath.as_str()).parent().unwrap().to_path_buf();
    for _ in 0..height_lim {
        let paths = fs::read_dir(target.to_str().unwrap())
            .unwrap()
            .into_iter()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap().path().to_str().unwrap().to_string())
            .collect_vec();
        let path = paths.into_iter().find(|x| x.ends_with("rustfmt.toml"));
        if let Some(path) = path {
            args.rustfmt_config = Some(path);
            break;
        } else {
            target = target.join("../");
        }
    }
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, ignore: &str, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() && (ignore == "" || dir.to_str().is_some_and(|x| !x.contains(ignore))) {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, ignore, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn main() {
    let args = CoccinelleForRust::parse();
    init_logger(&args);

    makechecks(&args);
    let targetpath = Path::new(&args.targetpath);
    if targetpath.is_file() {
        transformfile(&args);
    } else {
        let mut files = vec![];
        let _ = visit_dirs(targetpath, &args.ignore, &mut |file: &DirEntry| {
            if file.file_name().to_str().unwrap().ends_with(".rs") {
                files.push(String::from(file.path().to_str().unwrap()));
            }
        });
        transformfiles(&args, files);
    }
}
