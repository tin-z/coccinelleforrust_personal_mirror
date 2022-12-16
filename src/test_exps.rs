use ide_db::line_index::{LineIndex, LineCol};
use parser::SyntaxKind;
use syntax::{SyntaxNode, SyntaxToken, ast::{AnyHasArgList, Type}, AstNode};
use crate::wrap::{Rnode, position_info, info, wrap, mcodekind, Syntax, bef_aft};
use crate::visitor_ast0::ast0::worker;

impl Rnode {
    pub fn set_test_exps(&mut self, vec: &mut Vec<Rnode>, lindex: &LineIndex){
        self.wrapper.set_test_exps();
        vec.push(self.clone());
        match &self.astnode{
            Syntax::Node(node) => {
                match node.kind(){
                    parser::SyntaxKind::PAREN_EXPR => {
                        let mut wrap = fill_wrap(lindex, &node);
                        wrap.set_test_exps(vec, lindex);
                    }
                    _ => {}
                }
            }
            Syntax::Token(_token) => {
            }
        }
    }
}

pub fn wrap_keyword_aux(lindex: LineIndex, node: Option<SyntaxToken>) -> Option<Rnode> {
    match node {
        Some(node) => {
            let sindex: LineCol = lindex.line_col(node.text_range().start());
            let eindex: LineCol = lindex.line_col(node.text_range().end());

            let pos_info: position_info = position_info::new(
                sindex.line,
                eindex.line,
                0,
                0,
                sindex.col,
                node.text_range().start().into(),
            );
            let info = info::new(
                pos_info,
                false,
                false,
                vec![],
                vec![],
                vec![],
                vec![],
                false,
            );
            let kind = node.kind();
            let wrap: wrap = wrap::new(
                info,
                0,
                mcodekind::MIXED(),
                None,
                bef_aft {},
                AnyHasArgList::can_cast(kind),
                false,
                false,
                vec![],
            );

            Some(Rnode {
                wrapper: wrap,
                astnode: Syntax::Token(node),
                children: vec![],
            }); None
        }
        None => None,
    }
}

pub fn fill_wrap(lindex: &LineIndex, node: &SyntaxNode) -> Rnode{

    let sindex: LineCol = lindex.line_col(node.text_range().start());
    let eindex: LineCol = lindex.line_col(node.text_range().end());
    let mut nl: usize = 0;
    for s in  node.children_with_tokens(){
        s.as_token().map(
            |token|{
                if token.kind()==syntax::SyntaxKind::WHITESPACE {
                    nl+=token.to_string().matches('\n').count();
                }
            }
        ); 
    };
    let pos_info: position_info = position_info::new(
        sindex.line,
        eindex.line,
        sindex.line,
        eindex.line-(nl as u32),
        sindex.col,
        node.text_range().start().into(),
    );

    let info = info::new(
        pos_info,
        false,
        false,
        vec![],
        vec![],
        vec![],
        vec![],
        false,
    );
    let wrap: wrap = wrap::new(
        info,
        0,
        mcodekind::MIXED(),
        Type::cast(node.to_owned()),
        bef_aft {},
        false,
        false,
        false,
        vec![],
    );
    Rnode {
        wrapper: wrap,
        astnode: Syntax::Node(node.clone()),
        children: vec![],
    }
}

pub fn process_exp(exp: &mut Rnode){
    exp.wrapper.set_test_exps();
    match exp.astnode.kind(){
        SyntaxKind::PAREN_EXPR => {
            process_exp(&mut exp.children[1]);
        }
        _ => {}
    }
}

pub fn wrap_node_aux<'a>(
    worker: &mut worker<Rnode>,
    lindex: LineIndex,
    node: Box<&dyn AstNode>,
    df: &'a mut dyn FnMut(&mut worker<Rnode>) -> Vec<Rnode>,
) -> Option<Rnode> {
    let mut children = df(worker);
    let mut wrap = fill_wrap(&lindex, node.syntax());
    match node.syntax().kind(){
        SyntaxKind::IF_EXPR => {
            process_exp(&mut children[0]);
        }
        SyntaxKind::WHILE_EXPR => {
            process_exp(&mut children[0]);
        }
        _ => { }
    }
    wrap.set_children(children);
    Some(wrap)
}
