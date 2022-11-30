mod ast0;

use ast0::{Wrap, Info, position_info, token_info};
use syntax::ast::*;
use syntax::SourceFile;

fn parse(contents:&str){
    let root = SourceFile::parse(contents).tree();
    let mut lino = 1;//linenumber
    let mut cono = 1;//column number
    let wrap:Wrap;
    let info:Info;
    let position_info:position_info;
    let token_info:token_info;

    for item in root.syntax().children_with_tokens(){
        
        match item.as_node(){
            
            Some(node) => { 
                position_info = position_info{
                    line_start: lino,
                    line_end: lino + node.to_string().matches('\n').count(),
                    logical_start: 0,
                    logical_end: 0,
                    column: cono,
                    offset: 0//what is the offset?
                };

                info = Info{
                    pos_info: position_info,
                    attachable_start: false, attachable_end: false,
                    mcode_start: vec![], mcode_end: vec![],
                };

                wrap = Wrap{
                    node: node,
                };
                
         },
            None => {},
        }
        match item.as_token()
        {
            Some(token) => {
                lino += token.to_string().matches('\n').count(); 
                cono += token.to_string().len();
            },
            None => {}
        }
        
    }
}