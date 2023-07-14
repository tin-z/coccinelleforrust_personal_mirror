use std::fmt::Debug;
use std::process::exit;

use super::visitor_ast0::work_node;
use ide_db::line_index::{LineCol, LineIndex};
use parser::SyntaxKind;
use syntax::ast::Type;
use syntax::{SourceFile, SyntaxElement, SyntaxNode, SyntaxToken};

#[derive(PartialEq, Clone)]
/// Semantic Path Node
pub struct Snode {
    pub wrapper: Wrap,
    pub astnode: SyntaxElement,
    pub children: Vec<Snode>,
}

pub type Pluses = (Vec<Snode>, Vec<Snode>);

impl Debug for Snode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Snode")
            .field("astnode", &self.astnode.to_string())
            .field("children", &self.children)
            .finish()
    }
}

impl<'a> Snode {
    pub fn new_root(wrapper: Wrap, syntax: SyntaxElement, children: Vec<Snode>) -> Snode {
        Snode { wrapper: wrapper, astnode: syntax, children: children }
    }

    pub fn set_children(&mut self, children: Vec<Snode>) {
        self.children = children
    }

    pub fn tonode(self) -> SyntaxNode {
        self.astnode.into_node().unwrap()
    }

    pub fn toktoken(self) -> SyntaxToken {
        self.astnode.into_token().unwrap()
    }

    pub fn kind(&self) -> SyntaxKind {
        self.astnode.kind()
    }

    pub fn gettokenstream(&self) -> String {
        if self.children.len() == 0 {
            return String::from(self.astnode.to_string());
        } else {
            let mut tokens: String = String::new();
            for i in &self.children {
                tokens = format!("{} {}", tokens, i.gettokenstream());
            }
            return String::from(tokens.trim());
        }
    }

    fn print_tree_aux(&self, pref: &String) {
        println!("{}{:?}, {:?}", pref, self.kind(), self.wrapper.modkind);
        let mut newbuf = String::from(pref);
        newbuf.push_str(&String::from("--"));
        for child in &self.children {
            child.print_tree_aux(&newbuf)
        }
    }

    pub fn print_tree(&self) {
        //stticly debug function
        self.print_tree_aux(&String::from("--"));
    }

    pub fn isexpr(&self) -> bool {
        use SyntaxKind::*;

        match self.kind() {
            TUPLE_EXPR
            | ARRAY_EXPR
            | PAREN_EXPR
            | PATH_EXPR
            | CLOSURE_EXPR
            | IF_EXPR
            | WHILE_EXPR
            | LOOP_EXPR
            | FOR_EXPR
            | CONTINUE_EXPR
            | BREAK_EXPR
            | RETURN_EXPR
            | YIELD_EXPR
            | LET_EXPR
            | UNDERSCORE_EXPR
            | MACRO_EXPR
            | MATCH_EXPR
            | RECORD_EXPR
            | RECORD_EXPR_FIELD_LIST
            | RECORD_EXPR_FIELD
            | BOX_EXPR
            | CALL_EXPR
            | INDEX_EXPR
            | METHOD_CALL_EXPR
            | FIELD_EXPR
            | AWAIT_EXPR
            | TRY_EXPR
            | CAST_EXPR
            | REF_EXPR
            | PREFIX_EXPR
            | RANGE_EXPR
            | BIN_EXPR
            | EXPR_STMT
            | LITERAL
            | NAME_REF => true,
            _ => false,
        }
    }

    pub fn getdisjs(&'a self) -> (Vec<&'a Snode>, Pluses) {
        if !self.wrapper.isdisj {
            return (vec![], (vec![], vec![]));
        }
        fn collectdisjs<'b>(node: &'b Snode) -> Vec<&'b Snode> {
            //this function also returns the plus at the end of a disjunction
            let mut disjs: Vec<&Snode> = vec![];
            if node.wrapper.isdisj {
                disjs.push(&node.children[2].children[0]); //stmtlist is pushed
                match &node.children[..] {
                    [_ifkw, _cond, _block, _elsekw, ifblock] => {
                        disjs.append(&mut collectdisjs(ifblock));
                    }
                    _ => {}
                }
            }
            return disjs;
        }
        let disjs = collectdisjs(&self);
        return (disjs, (self.wrapper.plusesbef.clone(), self.wrapper.plusesaft.clone()));
    }
}


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MetavarName {
    pub rulename: String,
    pub varname: String,
}


#[derive(Clone, PartialEq)]
pub struct Dummy {}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum MODKIND {
    PLUS,
    MINUS,
}

#[derive(Clone, PartialEq)]
pub struct TokenInfo {
    tline_start: usize,
    tline_end: usize,
    left_offset: usize,
    right_offset: usize,
}

#[derive(Clone, PartialEq)]
pub struct PositionInfo {
    pub line_start: usize,
    pub line_end: usize,
    pub logical_start: usize,
    pub logical_end: usize,
    pub column: usize,
    pub offset: usize,
}

impl PositionInfo {
    pub fn new(
        line_start: usize,
        line_end: usize,
        logical_start: usize,
        logical_end: usize,
        column: usize,
        offset: usize,
    ) -> PositionInfo {
        PositionInfo {
            line_start: line_start,
            line_end: line_end,
            logical_start: logical_start,
            logical_end: logical_end,
            column: column,
            offset: offset,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Count {
    ONE,
    MINUS,
}

#[derive(PartialEq, Clone)]
pub enum Replacement {
    REPLACEMENT(Vec<Vec<Snode>>),
    NOREPLACEMENT,
}

#[derive(PartialEq, Clone)]
pub enum Befaft {
    BEFORE(Vec<Vec<Snode>>),
    AFTER(Vec<Vec<Snode>>),
    BEFOREAFTER(Vec<Vec<Snode>>, Vec<Vec<Snode>>),
    NOTHING,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Mcodekind {
    //MINUS(Replacement),
    PLUS(Count),
    //CONTEXT(Befaft),
    //MIXED(Befaft),
}

#[derive(Clone, PartialEq)]
pub struct DotsBefAft {}

#[derive(PartialEq, Clone)]
pub struct Info {
    pos_info: PositionInfo,
    attachable_start: bool,
    attachable_end: bool,
    mcode_start: Vec<Mcodekind>,
    mcode_end: Vec<Mcodekind>,
    strings_before: Vec<(Dummy, PositionInfo)>,
    strings_after: Vec<(Dummy, PositionInfo)>,
    is_symbol_ident: bool,
}

impl Info {
    pub fn new(
        pos_info: PositionInfo,
        attachable_start: bool,
        attachable_end: bool,
        mcode_start: Vec<Mcodekind>,
        mcode_end: Vec<Mcodekind>,
        strings_before: Vec<(Dummy, PositionInfo)>,
        strings_after: Vec<(Dummy, PositionInfo)>,
        is_symbol_ident: bool,
    ) -> Info {
        Info {
            pos_info: pos_info,
            attachable_start: attachable_start,
            attachable_end: attachable_end,
            mcode_start: mcode_start,
            mcode_end: mcode_end,
            strings_before: strings_before,
            strings_after: strings_after,
            is_symbol_ident,
        }
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum KeepBinding {
    UNITARY,    //Need no info
    NONUNITARY, //Need an env entry
    SAVED,      //Need a witness
}

type Minfo = (MetavarName, KeepBinding); //rulename, metavar name, keepbinding

#[derive(Clone, Hash, Debug, Eq)]
pub enum MetaVar {
    NoMeta,
    Exp(Minfo),
    Id(Minfo),
}

impl MetaVar {
    pub fn getname(&self) -> &str {
        match self {
            MetaVar::NoMeta => {
                panic!("Should never happen");
            }
            MetaVar::Id(minfo) => minfo.0.varname.as_str(),
            MetaVar::Exp(minfo) => minfo.0.varname.as_str(),
        }
    }

    pub fn gettype(&self) -> &str {
        match self {
            MetaVar::NoMeta => "None",
            MetaVar::Id(_minfo) => "identifier",
            MetaVar::Exp(_minfo) => "expression",
        }
    }

    pub fn setbinding(&mut self, binding: KeepBinding) {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => {
                minfo.1 = binding;
            }
            Self::Id(minfo) => {
                minfo.1 = binding;
            }
        }
    }

    pub fn getminfo(&self) -> &Minfo {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => &minfo,
            Self::Id(minfo) => &minfo,
        }
    }

    pub fn getrulename(&self) -> &str {
        match self {
            Self::NoMeta => {
                panic!("Should not occur.");
            }
            Self::Exp(minfo) => &minfo.0.rulename.as_str(),
            Self::Id(minfo) => &minfo.0.rulename.as_str(),
        }
    }

    pub fn new(rulename: &str, name: &str, ty: &str) -> MetaVar {
        let minfo = (
            MetavarName { rulename: rulename.to_string(), varname: name.to_string() },
            KeepBinding::UNITARY,
        );
        match ty {
            "expression" => MetaVar::Exp(minfo),
            "identifier" => MetaVar::Id(minfo),
            _ => MetaVar::NoMeta,
        }
    }

    pub fn isnotmeta(&self) -> bool {
        match self {
            MetaVar::NoMeta => true,
            _ => false,
        }
    }
}

impl PartialEq for MetaVar {
    fn eq(&self, other: &Self) -> bool {
        self.getname() == other.getname() && self.getrulename() == other.getrulename()
    }
}

#[derive(PartialEq, Clone)]
pub struct Wrap {
    info: Info,
    index: usize,
    pub mcodekind: Mcodekind,
    exp_ty: Option<Type>,
    bef_aft: DotsBefAft, //needed?
    pub plusesbef: Vec<Snode>,
    pub plusesaft: Vec<Snode>,
    pub metavar: MetaVar,
    true_if_arg: bool,
    pub true_if_test: bool,
    pub true_if_test_exp: bool,
    iso_info: Vec<(String, Dummy)>,
    pub isdisj: bool,
    pub modkind: Option<MODKIND>,
}

impl Wrap {
    pub fn new(
        info: Info,
        index: usize,
        mcodekind: Mcodekind,
        exp_ty: Option<Type>,
        bef_aft: DotsBefAft,
        plusesbef: Vec<Snode>,
        plusesaft: Vec<Snode>,
        metavar: MetaVar,
        true_if_arg: bool,
        true_if_test: bool,
        true_if_test_exp: bool,
        iso_info: Vec<(String, Dummy)>,
        isdisj: bool,
    ) -> Wrap {
        Wrap {
            info: info,
            index: index,
            mcodekind: mcodekind,
            exp_ty: exp_ty,
            bef_aft: bef_aft,
            plusesbef: plusesbef,
            plusesaft: plusesaft,
            metavar: metavar,
            true_if_arg: true_if_arg,
            true_if_test: true_if_test,
            true_if_test_exp: true_if_test_exp,
            iso_info: iso_info,
            isdisj: isdisj,
            modkind: None,
        }
    }

    pub fn is_ident(&self) -> bool {
        self.info.is_symbol_ident
    }

    pub fn getlinenos(&self) -> (usize, usize) {
        (self.info.pos_info.line_start, self.info.pos_info.line_end)
    }

    pub fn set_logilines_start(&mut self, lino: usize) {
        self.info.pos_info.logical_start = lino;
    }

    pub fn set_logilines_end(&mut self, lino: usize) {
        self.info.pos_info.logical_end = lino;
    }

    pub fn getlogilinenos(&self) -> (usize, usize) {
        (self.info.pos_info.logical_start, self.info.pos_info.logical_end)
    }

    pub fn setmodkind(&mut self, modkind: String) {
        match modkind.as_str() {
            "+" => self.modkind = Some(MODKIND::PLUS),
            "-" => self.modkind = Some(MODKIND::MINUS),
            _ => self.modkind = None,
        }
    }
}

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxElement) -> Wrap {
    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());

    let pos_info: PositionInfo = PositionInfo::new(
        //all casted to usize because linecol returns u32
        sindex.line as usize,
        eindex.line as usize,
        0,
        0,
        sindex.col as usize,
        node.text_range().start().into(),
    );

    let info = Info::new(pos_info, false, false, vec![], vec![], vec![], vec![], false);
    let wrap: Wrap = Wrap::new(
        info,
        0,
        Mcodekind::PLUS(Count::ONE),
        None, //will be filled later with type inference
        DotsBefAft {},
        vec![],
        vec![],
        MetaVar::NoMeta,
        false,
        false,
        false,
        vec![],
        false,
    );
    wrap
}

pub fn parsedisjs<'a>(node: &mut Snode) {
    //for later
    if node.kind() == SyntaxKind::IF_EXPR {
        //println!("does it come here");
        //let ifexpr: IfExpr = IfExpr::cast(node.astnode.into_node().unwrap()).unwrap();//Just checked above
        let cond = &node.children[1]; //this gets the node for condition
        if cond.kind() == SyntaxKind::PATH_EXPR && cond.astnode.to_string() == "COCCIVAR" {
            let block = &mut node.children[2].children[0].children;
            //println!("{:?}", block[0].kind());
            block.remove(0);
            block.remove(block.len() - 1);
            node.wrapper.isdisj = true;
            //println!("december slowly creeps into my eptember heart");
        }
    }
}

//for wrapping
pub fn wrap_root(contents: &str) -> Snode {
    let lindex = LineIndex::new(contents);

    let parse = SourceFile::parse(contents);
    let errors = parse.errors();

    if errors.len() > 0 {
        for error in errors {
            let lindex = lindex.line_col(error.range().start());
            println!("Error : {} at line: {}, col {}", error.to_string(), lindex.line, lindex.col);
            println!("{}", parse.syntax_node().to_string());
            exit(1);
        }
    }

    let root = SourceFile::parse(contents).syntax_node();
    let wrap_node = &|lindex: &LineIndex,
                      node: SyntaxElement,
                      modkind: Option<String>,
                      df: &dyn Fn(&SyntaxElement) -> Vec<Snode>|
     -> Snode {
        let mut wrapped = fill_wrap(&lindex, &node);
        wrapped.setmodkind(modkind.unwrap_or(String::new()));
        let children = df(&node);
        let mut snode = Snode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children: children,
        };
        parsedisjs(&mut snode);
        if snode.kind() == SyntaxKind::EXPR_STMT && snode.children.len() == 1 {
            // this means there is an expression statement without a ; at the ens
            //the reason these are removed because rust-analyzer seems to alter between
            //assigning ExprStmt and IfExprs(maybe others too)
            return snode.children.into_iter().next().unwrap();
        }
        snode
    };
    work_node(&lindex, wrap_node, SyntaxElement::Node(root), None)
}
