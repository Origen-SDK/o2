use crate::generator::PAT;
use crate::TEST;
use origen_metal::ast::Node;

pub fn cycle() {
    TEST.push(cycle_node())
}

pub fn cycle_node() -> Node<PAT> {
    node!(PAT::Cycle, 1, true)
}

pub fn repeat(cnt: u32) {
    TEST.push(repeat_node(cnt))
}

pub fn repeat_node(cnt: u32) -> Node<PAT> {
    node!(PAT::Cycle, cnt, true)
}

pub fn repeat2(cnt: u32, compressable: bool) {
    TEST.push(repeat2_node(cnt, compressable))
}

pub fn repeat2_node(cnt: u32, compressable: bool) -> Node<PAT> {
    node!(PAT::Cycle, cnt, compressable)
}
