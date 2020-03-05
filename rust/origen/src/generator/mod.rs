mod ast;
mod processor;
mod processors;
mod test_manager;

pub use test_manager::TestManager;

#[cfg(test)]
mod tests {
    use crate::generator::ast::*;
    use crate::generator::processors::*;
    use crate::TEST;

    #[test]
    fn nodes_can_be_created_and_nested() {
        TEST.start("trim_vbgap");
        let c = Node::new(Attrs::Comment("Hello".to_string()));
        TEST.push(c);

        let reg_trans = Node::new(Attrs::RegWrite(10, 0x12345678_u32.into()));
        let tid = TEST.push_and_open(reg_trans);
        let c = Node::new(Attrs::Comment(
            "Should be inside reg transaction".to_string(),
        ));
        TEST.push(c);
        let cyc = Node::new(Attrs::Cycle(1));
        for _i in 0..5 {
            TEST.push(cyc.clone());
        }
        println!("The AST in progress, we want to see the RegWrite here:");
        println!("{}", TEST.to_string());

        TEST.close(tid).expect("Closed reg trans properly");

        let c = Node::new(Attrs::Comment(
            "Should be outside reg transaction".to_string(),
        ));
        TEST.push(c);

        println!("The completed AST:");
        println!("{}", TEST.to_string());

        println!("The completed AST with upcased comments:");
        let new_ast = TEST.process(&|ast| UpcaseComments::run(ast));
        println!("{}", new_ast);

        println!("The completed AST with upcased comments and combined cycles:");
        let new_ast = CycleCombiner::run(&new_ast);
        println!("{}", new_ast);

        println!("Check the original AST is still available/unmodified:");
        println!("{}", TEST.to_string());
    }
}
