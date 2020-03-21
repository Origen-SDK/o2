#[macro_use]
pub mod ast;
pub mod processor;
mod processors;
mod test_manager;

pub use test_manager::TestManager;

#[cfg(test)]
mod tests {
    use crate::generator::ast::*;
    use crate::generator::processors::*;
    use crate::generator::TestManager;

    #[test]
    fn basic_ast_creation_and_processor_test() {
        let test = TestManager::new();

        test.start("trim_vbgap");
        let c = node!(Comment, 1, "Hello".to_string());
        test.push(c);

        let reg_trans = node!(RegWrite, 10, 0x12345678_u32.into(), None, None);
        let tid = test.push_and_open(reg_trans);
        let c = node!(Comment, 1, "Should be inside reg transaction".to_string());
        test.push(c);
        let cyc = node!(Cycle, 1, false);
        test.push(cyc);
        let cyc = node!(Cycle, 1, true);
        for _i in 0..5 {
            test.push(cyc.clone());
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
        assert_eq!(test, ast1);

        // Complete the AST and test again

        test.close(tid).expect("Closed reg trans properly");
        let c = node!(Comment, 1, "Should be outside reg transaction".to_string());
        test.push(c);

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
        assert_eq!(test, ast2);
        assert_eq!(ast2, test);

        // Test upcase comments processor

        let new_ast = test.process(&mut |ast| UpcaseComments::run(ast));

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
        assert_eq!(test, ast2);
    }

    #[test]
    fn nodes_can_be_replaced() {
        let test = TestManager::new();

        test.start("t1");
        let reg_trans = node!(RegWrite, 10, 0x12345678_u32.into(), None, None);
        test.push(node!(Cycle, 1, false));
        let _tid = test.push_and_open(reg_trans);
        test.push(node!(Cycle, 1, false));
        test.push(node!(Cycle, 1, true));
        test.push(node!(Cycle, 1, true));

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 1, true));
        assert_eq!(test, ast);

        // Test replacing the last node
        test.replace(node!(Cycle, 5, false), 0).expect("Ok1");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 5, false));
        assert_eq!(test, ast);

        // Test replacing with offset
        test.replace(node!(Cycle, 10, false), 2).expect("Ok2");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 10, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 5, false));
        assert_eq!(test, ast);

        // Test replacing an open node
        let test2 = test.clone();
        test2.replace(node!(Cycle, 15, true), 3).expect("Ok3");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 15, true));
        assert_eq!(test2, ast);

        // Test replacing an upstream node
        test.replace(node!(Cycle, 15, true), 4).expect("Ok4");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 15, true));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 10, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 5, false));
        assert_eq!(test, ast);

        test.replace(node!(Test, "t2".to_string()), 5).expect("Ok5");

        let mut ast = AST::new(node!(Test, "t2".to_string()));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 10, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 5, false));
        assert_eq!(test, ast);
    }

    #[test]
    fn nodes_can_be_inserted() {
        let test = TestManager::new();

        test.start("t1");
        let reg_trans = node!(RegWrite, 10, 0x12345678_u32.into(), None, None);
        test.push(node!(Cycle, 1, false));
        let _tid = test.push_and_open(reg_trans);
        test.push(node!(Cycle, 1, false));
        test.push(node!(Cycle, 1, true));
        test.push(node!(Cycle, 1, true));

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 1, true));
        assert_eq!(test, ast);

        // Test inserting the last node
        test.insert(node!(Cycle, 6, false), 0).expect("Ok1");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 6, false));
        assert_eq!(test, ast);

        // Test inserting within immediate children
        test.insert(node!(Cycle, 7, false), 2).expect("Ok2");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 7, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 6, false));
        assert_eq!(test, ast);

        test.insert(node!(Cycle, 8, false), 5).expect("Ok2");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 8, false));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 7, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 6, false));
        assert_eq!(test, ast);

        // Test inserting within next level up children

        test.insert(node!(Cycle, 9, false), 7).expect("Ok2");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 9, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 8, false));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 7, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 6, false));
        assert_eq!(test, ast);

        test.insert(node!(Cycle, 10, false), 9).expect("Ok2");

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 10, false));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 9, false));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 8, false));
        ast.push(node!(Cycle, 1, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 7, false));
        ast.push(node!(Cycle, 1, true));
        ast.push(node!(Cycle, 6, false));
        assert_eq!(test, ast);
    }

    #[test]
    fn nodes_can_be_inspected() {
        let test = TestManager::new();

        test.start("t1");
        test.push(node!(Cycle, 1, true));
        let reg_trans = node!(RegWrite, 10, 0x12345678_u32.into(), None, None);
        let _tid = test.push_and_open(reg_trans);
        test.push(node!(Cycle, 2, true));
        test.push(node!(Cycle, 3, true));
        test.push(node!(Cycle, 4, true));

        assert_eq!(test.get(0).unwrap(), node!(Cycle, 4, true));
        assert_eq!(test.get(1).unwrap(), node!(Cycle, 3, true));
        assert_eq!(test.get(2).unwrap(), node!(Cycle, 2, true));
        assert_eq!(test.get(4).unwrap(), node!(Cycle, 1, true));

        // Test cycle optimizer code
        if let Attrs::Cycle(repeat, compressable) = test.get(0).unwrap().attrs {
            if compressable {
                test.replace(node!(Cycle, repeat + 1, true), 0).expect("ok");
            }
        }

        let mut ast = AST::new(node!(Test, "t1".to_string()));
        ast.push(node!(Cycle, 1, true));
        let _r = ast.push_and_open(node!(RegWrite, 10, 0x12345678_u32.into(), None, None));
        ast.push(node!(Cycle, 2, true));
        ast.push(node!(Cycle, 3, true));
        ast.push(node!(Cycle, 5, true));
        assert_eq!(test, ast);
    }
}
