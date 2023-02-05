use crate::parsing_cocci::ast0::SNode;
use itertools::izip;

fn matcher(a: &SNode, b: &SNode) -> bool {
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