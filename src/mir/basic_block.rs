use super::node::*;

pub struct BasicBlock {
    pub id: u32,
    pub(super) nodes: Vec<Box<MIRNode>>,
    pub(super) terminator: Option<Box<MIRNode>>,
    pub(super) preds: Vec<u32>,
    pub(super) sucs: Vec<u32>,
}

impl BasicBlock {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            nodes: vec![],
            terminator: None,
            preds: vec![],
            sucs: vec![],
        }
    }
}
