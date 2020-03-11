#[macro_use]
pub mod ast;
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
    fn basic_ast_creation_and_processor_test() {
        TEST.start("trim_vbgap");
        let c = node!(Comment, 1, "Hello".to_string());
        TEST.push(c);

        let reg_trans = node!(RegWrite, 10, 0x12345678_u32.into(), None, None);
        let tid = TEST.push_and_open(reg_trans);
        let c = node!(Comment, 1, "Should be inside reg transaction".to_string());
        TEST.push(c);
        let cyc = node!(Cycle, 1, false);
        TEST.push(cyc);
        let cyc = node!(Cycle, 1, true);
        for _i in 0..5 {
            TEST.push(cyc.clone());
        }

        // Verify comparisons work

        let mut ast1 = AST::new(node!(Test, "trim_vbgap".to_string()));
        ast1.push(node!(Comment, 1, "Hello".to_string()));
        let r = ast1.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast1.push(node!(
            Comment,
            1,
            "Should be inside reg transaction".to_string()
        ));
        ast1.push(node!(Cycle, 1, false));
        let cyc = node!(Cycle, 1, true);
        for _i in 0..5 {
            ast1.push(cyc.clone());
        }
        ast1.close(r).unwrap();
        assert_eq!(TEST, ast1);

        // Complete the AST and test again

        TEST.close(tid).expect("Closed reg trans properly");
        let c = node!(Comment, 1, "Should be outside reg transaction".to_string());
        TEST.push(c);

        let mut ast2 = AST::new(node!(Test, "trim_vbgap".to_string()));
        ast2.push(node!(Comment, 1, "Hello".to_string()));
        let r = ast2.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast2.push(node!(
            Comment,
            1,
            "Should be inside reg transaction".to_string()
        ));
        ast2.push(node!(Cycle, 1, false));
        let cyc = node!(Cycle, 1, true);
        for _i in 0..5 {
            ast2.push(cyc.clone());
        }
        ast2.close(r).unwrap();
        ast2.push(node!(
            Comment,
            1,
            "Should be outside reg transaction".to_string()
        ));
        assert_eq!(TEST, ast2);
        assert_eq!(ast2, TEST);

        // Test upcase comments processor

        let new_ast = TEST.process(&|ast| UpcaseComments::run(ast));

        let mut ast = AST::new(node!(Test, "trim_vbgap".to_string()));
        ast.push(node!(Comment, 1, "HELLO".to_string()));
        let r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(
            Comment,
            1,
            "SHOULD BE INSIDE REG TRANSACTION".to_string()
        ));
        ast.push(node!(Cycle, 1, false));
        let cyc = node!(Cycle, 1, true);
        for _i in 0..5 {
            ast.push(cyc.clone());
        }
        ast.close(r).unwrap();
        ast.push(node!(
            Comment,
            1,
            "SHOULD BE OUTSIDE REG TRANSACTION".to_string()
        ));
        assert_eq!(new_ast, ast);
        assert_eq!(ast, new_ast);

        // Test cycle combiner processor

        let new_ast = CycleCombiner::run(&new_ast);

        let mut ast = AST::new(node!(Test, "trim_vbgap".to_string()));
        ast.push(node!(Comment, 1, "HELLO".to_string()));
        let r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(
            Comment,
            1,
            "SHOULD BE INSIDE REG TRANSACTION".to_string()
        ));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 5, true));
        ast.close(r).unwrap();
        ast.push(node!(
            Comment,
            1,
            "SHOULD BE OUTSIDE REG TRANSACTION".to_string()
        ));
        assert_eq!(new_ast, ast);

        // Test the original AST is still available/unmodified
        assert_eq!(TEST, ast2);
    }
}
