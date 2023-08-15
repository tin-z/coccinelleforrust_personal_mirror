// SPDX-License-Identifier: GPL-2.0

use clap::Parser;
use coccinelleforrust::commons::info::ParseError::{self, *};
use coccinelleforrust::commons::util::workrnode;
use coccinelleforrust::parsing_cocci::parse_cocci::processcocci;
use coccinelleforrust::parsing_rs::parse_rs::{processrs, processrswithsemantics};
use coccinelleforrust::parsing_rs::type_inference::gettypedb;
use coccinelleforrust::{
    engine::cocci_vs_rs::MetavarBinding, engine::transformation,
    interface::interface::CoccinelleForRust, parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode,
};
use env_logger::{Builder, Env};
use itertools::{izip, Itertools};
use ra_hir::{HirDisplay, Name, Semantics, Struct};
use ra_syntax::{ast, AstNode};
use ra_vfs::VfsPath;
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
        line.map(|sline| {
            if childa.wrapper.info.eline == sline {
                childa.wrapper.wspaces = childb.wrapper.wspaces.clone();
            } else {
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

        let mut fcommand = Command::new("rustfmt")
            .arg("--config-path")
            .arg(cfr.rustfmt_config.as_str())
            .arg(&randrustfile)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("rustfmt failed");

        if let Ok(_) = fcommand.wait() {
        } else {
            println!("Formatting failed.");
        }
    }

    let formattednode =
        processrs(&fs::read_to_string(&randrustfile).expect("Counld not read")).unwrap();
    adjustformat(transformedcode, &formattednode, None);

    transformedcode.writetreetofile(&randrustfile);
    //fs::write(&randrustfile, original.clone()).expect("Could not write file.");

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

    return (formatted, diffed);
    //}

    //transformedcode.writetreetofile(&randrustfile);
    //let transformed = fs::read_to_string(&randrustfile).expect("Could not read generated file");
    //fs::remove_file(randrustfile).expect("Could not reove file");

    //return (transformedcode.gettokenstream(), String::new());
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
}

fn transformfiles(args: &CoccinelleForRust, files: Vec<String>) {
    let patchstring = fs::read_to_string(&args.coccifile).expect("Could not read file.");
    let (_, needsti, _) = processcocci(&patchstring);
    //let lockedrules = Arc::new(Mutex::new(rules));
    if !needsti {
        let transform = |targetpath: &String| {
            //rule cloned here because to pass references while keeping
            //the code thread safe, I need to study more about locks and Mutex
            //let rules = Arc::clone(&lockedrules);
            //let rules = rules.lock().unwrap();

            let (rules, _, hasstars) = processcocci(&patchstring);
            //Currently have to parse cocci again because Rule has SyntaxNode which which has
            //rowan `NonNull<rowan::cursor::NodeData>` which cannot be shared between threads safely
            let rcode = fs::read_to_string(&targetpath).expect("Could not read file");
            let transformedcode = transformation::transformfile(&rules, rcode);

            let mut transformedcode = match transformedcode {
                Ok(node) => node,
                Err(error) => {
                    //failedfiles.push((error, targetpath));
                    match error {
                        TARGETERROR(msg, _) => println!("{}", msg),
                        RULEERROR(msg, error, _) => println!("{}:{}", msg, error),
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
            //println!();
            let (rules, needsti, hasstars) = processcocci(&patchstring);
            let fileid = vfs
                .file_id(&VfsPath::new_real_path(targetpath.clone()))
                .expect(&format!("Could not get FileId for file {}", &targetpath));
            let syntaxnode = semantics.parse_or_expand(fileid.into());
            let rcode = fs::read_to_string(targetpath).expect("Could not read file");

            let mut rnode = processrswithsemantics(&rcode, syntaxnode)
                .expect("Could not convert SyntaxNode to Rnode");
            workrnode(&mut rnode, &mut |node| {
                let node = if node.astnode().is_none() {
                    return false;
                } else {
                    node.astnode().unwrap()
                };

                let ty = ast::Expr::cast(node.clone())
                    .and_then(|ex| semantics.type_of_expr(&ex.into()))
                    .map(|ex| ex.original);
                let _ = ty.is_some_and(|ty| {
                    ty.as_adt().is_some_and(|x| {
                        let a = x.module(semantics.db).path_to_root(semantics.db)
                        .into_iter()
                        .rev()
                        .flat_map(|it| it.name(db).map(|name| name.display(db).to_string())).join("::");
                        let typename = x.module(semantics.db).display(semantics.db).to_string();
                        //let typename = x.display(semantics.db).to_string();
                        //let typename = x.name(semantics.db);
                        println!("{}: {}-{:?}", node.to_string(), a, typename);
                        true
                    }
                )
                });


                true
            });
            //transformrnode(&rules, rnode);
        };

        if !args.no_parallel {
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
            args.rustfmt_config = path;
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
        transformfiles(&args, vec![args.targetpath.to_string()]);
    } else {
        let mut files = vec![];
        let _ = visit_dirs(targetpath, &args.ignore, &mut |f: &DirEntry| {
            if f.file_name().to_str().unwrap().ends_with(".rs") {
                files.push(String::from(f.path().to_str().unwrap()));
            }
        });
        transformfiles(&args, files);
    }
}
