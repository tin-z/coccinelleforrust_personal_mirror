// SPDX-License-Identifier: GPL-2.0


//want to do some tests before removing these
fn ismetavar(rule: &mut Rule, node: &mut Rnode) -> metatype {
    let varname = node.astnode.to_string();
    for var in &rule.metavars {
        if varname.eq(&var.varname) {
            return var.metatype;
        }
    }
    metatype::NoMeta
}

fn flag_metavars(rule: &mut rule, node: &mut Rnode) {
    for mut child in node.children_with_tokens.iter_mut() {
        match (child.kind(), ismetavar(rule, child)) {
            (Tag::PATH_EXPR, a) => {
                child.wrapper.metatype = a;
            }
            _ => {
                flag_metavars(rule, child);
            }
        }
    }
}
