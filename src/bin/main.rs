use clap::Parser;
use coccinelleforrust::{
    commons::util::{getstmtlist, visitrnode, worksnode},
    engine::{disjunctions::getdisjunctions, transformation},
    engine::{
        cocci_vs_rs::{Looper, MetavarBinding},
        disjunctions::Disjunction,
        transformation::transform,
    },
    parsing_cocci::ast0::Snode,
    parsing_cocci::{ast0::MODKIND, parse_cocci::processcocci},
    parsing_rs::{ast_rs::Rnode, parse_rs::processrs},
};
use itertools::Itertools;
use rand::Rng;
use std::process::Command;
use std::{
    fs,
    path::{self, Path},
    process::exit,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CoccinelleForRust {
    /// Path of Semantic Patch File path
    #[arg(short, long)]
    coccifile: String,

    /// Path of Rust Target file path
    #[arg(short, long)]
    targetpath: String,

    /// Path of transformed file path
    #[arg(short, long)]
    output: Option<String>,

    /// rustfmt config file path
    #[arg(short, long, default_value_t = String::from("rustfmt.toml"))]
    rustfmt_config: String,
}

fn tokenf<'a>(node1: &'a Snode, node2: &'a Rnode) -> Vec<MetavarBinding<'a>> {
    // this is
    // Tout will have the generic types in itself
    // ie  ('a * 'b) tout //Ocaml syntax
    // Should I replace Snode and Rnode with generic types?
    // transformation.ml's tokenf
    // info_to_fixpos
    vec![]
}

fn transformfile(args: &CoccinelleForRust) {
    let patchstring = fs::read_to_string(args.coccifile.as_str()).expect("This shouldnt be empty");
    let rustcode = fs::read_to_string(args.targetpath.as_str()).expect("This shouldnt be empty");
    let mut rng = rand::thread_rng();

    let transformedcode = transformation::transformfile(patchstring, rustcode);
    let randfilename = format!("tmp{}.rs", rng.gen::<u32>());
    transformedcode.writetreetofile(&randfilename);
    Command::new("rustfmt")
        .arg("--config-path")
        .arg(args.rustfmt_config.as_str())
        .arg(&randfilename)
        .output()
        .expect("rustfmt failed");

    let data = fs::read_to_string(&randfilename).expect("Unable to read file");
    println!("After Formatting:\n\n{}", data);

    fs::remove_file(&randfilename).expect("No file found.");

    if let Some(outputfile) = &args.output {
        if let Err(written) = fs::write(outputfile, data) {
            eprintln!("Error in writing file.\n{:?}",written);
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

fn findfmtconfig(args: &mut CoccinelleForRust) {
    let height_lim: usize = 5;

    let mut target = Path::new(args.targetpath.as_str()).parent().unwrap().to_path_buf();
    for i in 0..height_lim {
        let mut paths = fs::read_dir(target.to_str().unwrap())
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

fn main() {
    let args = CoccinelleForRust::parse();

    makechecks(&args);
    transformfile(&args);
}
