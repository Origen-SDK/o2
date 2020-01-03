//! Maybe there's an easier way to do this. But, I don't see a way to have an Arena cleared
//! and I'm not sure that dropping a static variable is a thing.
use id_arena::Arena;
use super::ast_node::AstNode;

pub struct ArenaWrapper {
    cheat: Vec <Arena::<AstNode>>,
}

impl ArenaWrapper {
    pub fn new() -> ArenaWrapper {
        let mut aw = ArenaWrapper { cheat: Vec::new() };
        aw.cheat.push(Arena::<AstNode>::new());
        aw
    }
    
    pub fn clear(&mut self) {
        let cls = self.cheat.pop();
        drop(cls);
        self.cheat.push(Arena::<AstNode>::new());
    }
    
    pub fn arena(&mut self) -> &mut Arena::<AstNode> {
        &mut self.cheat[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn clear_works(){
        let mut pattern_arena = ArenaWrapper::new();
        assert_eq!(pattern_arena.arena().len(), 0);
        pattern_arena.arena().alloc(AstNode::Timeset("tp0".to_string()));
        assert_eq!(pattern_arena.arena().len(), 1);
        pattern_arena.clear();
        assert_eq!(pattern_arena.arena().len(), 0);
    }
}