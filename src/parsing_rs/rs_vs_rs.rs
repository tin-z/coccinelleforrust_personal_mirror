use crate::parsing_cocci::ast0::Snode;
use itertools::izip;

fn matcher(a: &Snode, b: &Snode) -> bool {
    match ( a.kind(), b.kind()) {
        _ => {
            for (e1, e2) in izip!(&a.children, &b.children) {
                if !matcher(e1, e2) {
                    return false
                }
            }
            true
        }
    }
}