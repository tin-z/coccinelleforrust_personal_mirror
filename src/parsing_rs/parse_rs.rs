pub fn processrs(contents: &str) -> Snode {
    let lindex = LineIndex::new(contents);
    let root = SourceFile::parse(contents).tree();
    let wrap_node = &|node: SyntaxElement, df: &dyn Fn(&SyntaxElement) -> Vec<Snode>| -> Snode {
        let wrapped = fill_wrap(&lindex, &node);
        let children = df(&node);
        let rnode = Snode {
            wrapper: wrapped,
            astnode: node, //Change this to SyntaxElement
            children: children,
        };
        rnode
    };
    work_node(wrap_node, SyntaxElement::Node(root.syntax().clone()))
}