mod ast;
mod processor;
mod processors;
mod test_manager;

pub use test_manager::TestManager;

#[cfg(test)]
mod tests {
    use crate::generator::ast::*;
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
        for _i in 0..10 {
            TEST.push(cyc.clone());
        }
        TEST.close(tid).expect("Closed reg trans properly");

        let c = Node::new(Attrs::Comment(
            "Should be outside reg transaction".to_string(),
        ));
        TEST.push(c);

        println!("{}", TEST.to_string());

        //let reg_write =
        //let mut reg_write =
        assert_eq!(1, 1);
    }
}
