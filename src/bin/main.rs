// SPDX-License-Identifier: GPL-2.0

use clap::Parser;
use coccinelleforrust::commons::info::ParseError::*;
use coccinelleforrust::commons::util::attach_spaces_right;
use coccinelleforrust::debugcocci;
use coccinelleforrust::parsing_cocci::parse_cocci::processcocci;
use coccinelleforrust::parsing_rs::parse_rs::{processrs, processrswithsemantics};
use coccinelleforrust::parsing_rs::type_inference::{gettypedb, set_types};
use coccinelleforrust::{
    engine::cocci_vs_rs::MetavarBinding, engine::transformation,
    interface::interface::CoccinelleForRust, parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode,
};
use env_logger::{Builder, Env};
use itertools::{izip, Itertools};
use ra_hir::Semantics;
use ra_vfs::VfsPath;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::fs::{canonicalize, DirEntry};
use std::io;
use std::io::Write;
use std::process::{Command, Output};
use std::{fs, path::Path, process::exit};
use tempfile::NamedTempFile;

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
    if line.is_some() {
        //eprintln!("{:?}", line);
        //eprintln!("{} here", node1.getunformatted());
    }

    if node1.wrapper.wspaces.0.contains("/*COCCIVAR*/") {
        node1.wrapper.wspaces = node2.wrapper.wspaces.clone();
        line = Some(node1.wrapper.info.sline);
        debugcocci!("Formatting line {}", line.unwrap());
    }
    let mut prev_space = String::new();
    let mut preva = None;
    for (childa, childb) in izip!(&mut node1.children, &node2.children) {
        line.map(|sline| {
            //eprintln!("{}, {:?}=={:?}", sline, childa.getunformatted(), childb.getunformatted());
            if childa.wrapper.info.eline == sline {
                childa.wrapper.wspaces = childb.wrapper.wspaces.clone();
            } else {
                line = None;
            }
        });
        line = adjustformat(childa, &childb, line);
        debugcocci!("Formatting line {}", line.unwrap());
        if line.is_some() {
            if preva.is_some() {
                attach_spaces_right(preva.unwrap(), prev_space.clone());
                preva = None;
                prev_space = String::new();
            }
        } else {
            preva = Some(childa);
            prev_space = childb.get_spaces_right();
        }
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
    //eprintln!("Dealing with - {}", targetpath);

    let tp = std::path::Path::new(targetpath);
    let parent = tp.parent().unwrap_or(std::path::Path::new("/tmp"));

    //-let randrustfile = format!("{}/tmp{}.rs", parent.display(), rng.gen::<u32>());
    let dirpath = parent.to_str().expect("Cannot get directory");
    let mut randfile =
        tempfile::Builder::new().tempfile_in(dirpath).expect("Cannot create temporary file.");
    let mut randrustfile = randfile.path().to_str().expect("Cannot get temporary file.");

    // In all cases, write the transformed code to a file so we can diff
    //VERY IMPORTANT :-
    //CHECK TO REMOVE THIS FILE FOR ALL ERROR CASES
    transformedcode.writetotmpnamedfile(&randfile);

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

        let output: Output;
        let mut core_command = Command::new("rustfmt");
        if let Some(fmtconfig_path) = &cfr.rustfmt_config {
            let fmtconfp = format!("--config-path {}", fmtconfig_path.as_str());
            output = core_command
                .arg(&randrustfile)
                .arg("--edition")
                .arg("2021")
                .arg(fmtconfp)
                .output()
                .expect("rustfmt failed");
        } else {
            output = core_command
                .arg(&randrustfile)
                .arg("--edition")
                .arg("2021")
                .output()
                .expect("rustfmt failed");
        }

        if !output.status.success() {
            if cfr.show_fmt_errors {
                eprint!(
                    "RUSTFMT ERR - {}",
                    String::from_utf8(output.stderr).unwrap_or(String::from("NONE"))
                );
            }
        }
        //if let Some(fmtconfig_path) = &cfr.rustfmt_config {
        //  fcommand = fcommand.arg("--config-path").arg(fmtconfig_path.as_str());
        //}

        //if fcommand.spawn().expect("rustfmt failed").wait().is_err() {
        //  eprintln!("Formatting failed.");
        //}
        let formattednode =
            match processrs(&fs::read_to_string(&randrustfile).expect("Could not read")) {
                Ok(rnode) => rnode,
                Err(_) => {
                    panic!("Cannot parse temporary file.");
                }
            };

        //let formattednode =
        //  processrs(&fs::read_to_string(&randrustfile).expect("Could not read")).unwrap();

        //eprintln!("{}", formattednode.getunformatted());
        adjustformat(transformedcode, &formattednode, None);
        randfile = NamedTempFile::new().expect("Cannot create temporary file.");
        randrustfile = randfile.path().to_str().expect("Cannot get temporary file.");
        transformedcode.writetotmpnamedfile(&randfile);
    }

    let diffed = if !cfr.suppress_diff {
        let diffout = Command::new("diff")
            .arg("-u")
            .arg("--text")
            .arg(targetpath)
            .arg(&randrustfile)
            .output()
            .expect("diff failed");

        String::from_utf8(diffout.stdout).expect("Bad diff")
    } else {
        String::new()
    };

    let formatted = fs::read_to_string(&randrustfile).expect("Unable to read file");

    (formatted, diffed)
}

fn showdiff(
    args: &CoccinelleForRust,
    transformedcode: &mut Rnode,
    targetpath: &str,
    hasstars: bool,
) {
    let (data, diff) = getformattedfile(&args, transformedcode, &targetpath);
    if !hasstars {
        if !args.suppress_diff {
            if !diff.is_empty() {
                println!("{}", diff);
            }
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
                println!("*{}", &line[1..]);
            } else {
                println!("{}", line)
            }
        }
    }
}

fn transformfiles(args: &CoccinelleForRust, files: &[String]) {
    let patchstring = fs::read_to_string(&args.coccifile).expect("Could not read file.");
    let (_, needsti, _) = processcocci(&patchstring);

    if !needsti {
        let transform = |targetpath: &String| {
            //eprintln!("Processing {}", targetpath);
            let (rules, _, hasstars) = processcocci(&patchstring);
            //Currently have to parse cocci again because Rule has SyntaxNode which which has
            //rowan `NonNull<rowan::cursor::NodeData>` which cannot be shared between threads safely
            // let files_tmp = do_get_files(&args, &args.targetpath, &rules);
            // eprintln!("{:?}", files_tmp);

            let rcode = fs::read_to_string(&targetpath)
                .expect(&format!("{} {}", "Could not read file", targetpath));
            let transformedcode = transformation::transformfile(&rules, rcode);
            let mut transformedcode = match transformedcode {
                Ok(node) => node,
                Err(error) => {
                    //failedfiles.push((error, targetpath));
                    match error {
                        TARGETERROR(msg, _) => eprintln!("{}", msg),
                        RULEERROR(msg, error, _) => {
                            println!("Transformation Error at rule {} : {}", msg, error)
                        }
                    }
                    println!("Failed to transform {}", targetpath);
                    return;
                }
            };
            showdiff(args, &mut transformedcode, targetpath, hasstars);
        };

        if !args.no_parallel {
            files.par_iter().for_each(transform);
        } else {
            files.iter().for_each(transform);
        }
    } else {
        if files.len() == 0 {
            return;
        }

        let (host, vfs) = gettypedb(&files[0]);
        let db = host.raw_database();
        //let semantics = &mut Semantics::new(db);
        let semantics = &Semantics::new(db);

        let transform = |targetpath: &String| {
            let (rules, _needsti, hasstars) = processcocci(&patchstring);
            let fileid = vfs
                .file_id(&VfsPath::new_real_path(targetpath.clone()))
                .expect(&format!("Could not get FileId for file {}", &targetpath));
            let syntaxnode = semantics.parse_or_expand(fileid.into());
            let rcode = fs::read_to_string(targetpath).expect("Could not read file");

            let mut rnode = processrswithsemantics(&rcode, syntaxnode)
                .expect("Could not convert SyntaxNode to Rnode");

            set_types(&mut rnode, semantics, db);

            let transformedcode = transformation::transformrnode(&rules, rnode);

            let mut transformedcode = match transformedcode {
                Ok(node) => node,
                Err(error) => {
                    //failedfiles.push((error, targetpath));
                    match error {
                        TARGETERROR(msg, _) => eprintln!("{}", msg),
                        RULEERROR(msg, error, _) => eprintln!("{}:{}", msg, error),
                    }
                    println!("Failed to transform {}", targetpath);
                    return;
                }
            };

            showdiff(args, &mut transformedcode, targetpath, hasstars);
            //transformrnode(&rules, rnode);
        };

        if !args.no_parallel {
            todo!("Parallel for type inference has not been implemented.");
            //files.par_iter().for_each(transform);
        } else {
            files.iter().for_each(transform);
        }
    };
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
        transformfiles(&args, &[String::from(canonicalize(targetpath).unwrap().to_str().unwrap())]);
    } else {
        let mut files = vec![];
        let _ = visit_dirs(targetpath, &args.ignore, &mut |f: &DirEntry| {
            if f.file_name().to_str().unwrap().ends_with(".rs") {
                files.push(String::from(canonicalize(f.path()).unwrap().to_str().unwrap()));
            }
        });
        transformfiles(&args, &files[..]);
    }
}
