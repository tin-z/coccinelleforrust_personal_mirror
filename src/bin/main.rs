use clap::Parser;
use coccinelleforrust::commons::info::ParseError::*;
use coccinelleforrust::{
    engine::cocci_vs_rs::MetavarBinding, engine::transformation,
    interface::interface::CoccinelleForRust, parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode,
};
use env_logger::{Builder, Env};
use itertools::Itertools;
use rand::Rng;
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
    if args.debug_cocci {
        options.push_str(
            "
            coccinelleforrust::parsing_cocci,
            coccinelleforrust::commons
        ",
        );
    }
    let env = Env::default().default_filter_or(&options);

    Builder::from_env(env)
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();
}

fn getformattedfile(
    cfr: &CoccinelleForRust,
    transformedcode: &Rnode,
    targetpath: &str,
) -> (String, String) {
    let mut rng = rand::thread_rng();
    let randrustfile = format!("tmp{}.rs", rng.gen::<u32>());

    let original = fs::read_to_string(&targetpath).expect("Unable to read file");

    transformedcode.writetreetofile(&targetpath);
    let mut fcommand = Command::new("rustfmt")
        .arg("--config-path")
        .arg(cfr.rustfmt_config.as_str())
        .arg(&targetpath)
        //.stdout(std::process::Stdio::null())
        //.stderr(std::process::Stdio::null())
        .spawn()
        .expect("rustfmt failed");

    if let Ok(a) = fcommand.wait() {
        println!("Formatting success :- {}", a.success());
    } else {
        println!("Formatting failed.");
    }

    fs::write(&randrustfile, original.clone()).expect("Could not write file.");

    let diffout = Command::new("git")
        .arg("diff")
        .arg("--no-index")
        .arg("--diff-algorithm=histogram")
        .arg(&randrustfile)
        .arg(targetpath)
        .output()
        .expect("diff failed");

    let formatted = fs::read_to_string(&targetpath).expect("Unable to read file");
    let diffed: String = String::from_utf8(diffout.stdout).expect("Bad diff");

    fs::write(targetpath, original).expect("Could not write file.");
    fs::remove_file(&randrustfile).expect("No file found.");

    return (formatted, diffed);
}

fn transformfiles(args: &CoccinelleForRust, files: Vec<String>) {
    for targetpath in files {
        let patchstring =
            fs::read_to_string(args.coccifile.as_str()).expect("This shouldnt be empty");
        let rustcode = fs::read_to_string(targetpath.as_str()).expect("This shouldnt be empty");

        let transformedcode = transformation::transformfile(patchstring, rustcode);

        let (transformedcode, hasstars) = match transformedcode {
            Ok(node) => node,
            Err(TARGETERROR(errors, file)) => {
                eprintln!("Error in reading target file.\n{}", errors);
                eprintln!("Unparsable file:\n{}", file);
                panic!();
            }
            Err(RULEERROR(rulename, errors, file)) => {
                eprintln!("Error in applying rule {}", rulename);
                eprintln!("Error:\n{}", errors);
                eprintln!("Unparsable file:\n{}", file);
                panic!();
            }
        };
        let (data, diff) = getformattedfile(&args, &transformedcode, &targetpath);
        if !hasstars {
            println!("{}", diff);

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
}

fn transformfile(args: &CoccinelleForRust) {
    let patchstring = fs::read_to_string(args.coccifile.as_str()).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(args.targetpath.as_str()).expect("This shouldnt be empty");

    let transformedcode = transformation::transformfile(patchstring, rustcode);

    let (transformedcode, hasstars) = match transformedcode {
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
    let (data, diff) = getformattedfile(&args, &transformedcode, &args.targetpath);
    if !hasstars {
        println!("{}", diff);
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
            args.rustfmt_config = path;
            break;
        } else {
            target = target.join("../");
        }
    }
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
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
        let _ = visit_dirs(targetpath, &mut |file: &DirEntry| {
            if file.file_name().to_str().unwrap().ends_with(".rs") {
                files.push(String::from(file.path().to_str().unwrap()));
            }
        });
        transformfiles(&args, files);
    }
}
