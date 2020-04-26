pub mod op;
use op::*;

pub struct BytecodeAssembler {
    pub code: Vec<u8>,
    pub feedback_slots: usize,
}

impl BytecodeAssembler {
    pub fn new() -> Self {
        Self {
            code: vec![],
            feedback_slots: 0,
        }
    }
    pub fn write_i32(&mut self, i: i32) {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(i) };
        self.code.extend(&bytes);
    }
    pub fn lda_int(&mut self, i: i32) {
        self.code.push(Op::LdaInt as u8);
        self.write_i32(i);
    }

    pub fn ret(&mut self) {
        self.code.push(Op::Return as u8);
    }
}
