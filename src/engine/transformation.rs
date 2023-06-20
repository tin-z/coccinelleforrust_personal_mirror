use crate::{parsing_cocci::ast0::Snode, parsing_rs::ast_rs::Rnode, commons::util::workrnode};

use super::cocci_vs_rs::MetavarBindings;

pub fn transform(mut rnode: Rnode, bindings: MetavarBindings) {
    let mut f = |x: &mut Rnode| {
        for i in bindings.minuses.clone().into_iter().flatten() {
            //would be better if I could use hashmaps
            if std::ptr::eq(x, i) {
                println!("hello");
            }
        }
    };
    workrnode(&mut rnode, &mut f);
}