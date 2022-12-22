pub mod visitor_ast0;
pub mod wrap;
pub mod parse_cocci;
mod util;

fn cocciffn() -> bool{
    true || (false || (true && false))
}