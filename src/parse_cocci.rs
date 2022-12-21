use crate::{
    make_parsable::make_parsable,
    wrap::{mcodekind, wrap_root, Rnode},
};
use parser::SyntaxKind;
use syntax::{
    ast::{BlockExpr, Fn, HasModuleItem},
    AstNode, SourceFile, SyntaxNode,
};

type Tag = SyntaxKind;

fn set_plus(node: &mut Rnode) {
    node.wrapper.mcodekind = mcodekind::PLUS();
}

fn get_blxpr(contents: &str) -> BlockExpr {
    let node = SourceFile::parse(contents).tree();
    let fnode = node.syntax().children().nth(0).unwrap();
    Fn::cast(fnode).unwrap().body().unwrap()
}

fn parsestmts(contents: &str, stmtnos: Vec<usize>, mut fname: String) -> (BlockExpr, BlockExpr) {
    //BlockExprs here should always be two block expressions containing the plus and minus statements respectively
    //Should this be replaces with SyntaxNodes?
    if fname == "" {
        fname = String::from("coccifn");
    }
    let mut plusstmts = String::from("");
    let mut minusstmts = String::from("");
    let lines: Vec<String> = contents.lines().map(String::from).collect();
    for lineno in stmtnos {
        let line = &lines[lineno];
        match line.chars().nth(0) {//same as .next()
            Some('+') => {
                plusstmts.push_str(line.as_str());
                plusstmts.push('\n');
            }
            Some('-') => {
                minusstmts.push_str(line.as_str());
                minusstmts.push('\n');
            }
            _ => {}
        }
    }

    println!("{:?}", plusstmts);
    let mut plusfn = format!("fn {}_plus {{ {} }}", fname, plusstmts);
    let mut minusfn = format!("fn {}_plus {{ {} }}", fname, minusstmts);

    (get_blxpr(&plusfn[..]), get_blxpr(&minusfn[..]))
}

pub fn parse_cocci(contents: &str) {
    let mut lines: Vec<String> = contents.lines().map(String::from).collect();
    let (_, linenos) = make_parsable(contents);
    let mut anos = 0; //used to keep track of rule

    //will temporarily store all statements 
    //in a rule before passing them off for processing
    let mut stmtnos: Vec<usize> = vec![];
    for lineno in linenos {
        //should never fail because line nos calculated in make_parsable
        let line = &lines[lineno]; 
        match line.chars().nth(0) {
            Some('@') => {
                if anos == 2 {
                    let (plusnodes, minusnodes) = parsestmts(contents, stmtnos, String::from(""));
                    stmtnos = vec![];
                    anos = 1;
                } else {
                    anos += 1;
                }
            }
            Some('+') => {
                stmtnos.push(lineno);
            }
            Some('-') => {
                stmtnos.push(lineno);
            }
            _ => { }
        }
    }
    let (plusnodes, minusnodes) = parsestmts(contents, stmtnos, String::from(""));//to take care of the last rule
}
